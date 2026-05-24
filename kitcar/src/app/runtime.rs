//! Runtime: the [`run`] entry point that drives an [`crate::App`] against a
//! live connection, and the [`Sender`] back-channel that hands packets/events
//! to the runtime from anywhere.
//!
//! [`run`] consumes an [`crate::App`] together with an `insim::Builder`,
//! opens the connection, emits a one-shot [`crate::Startup`] synthetic event,
//! then runs the dispatch loop until the connection drops or the back-channel
//! closes.

use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use futures::stream::{FuturesUnordered, StreamExt};
use indexmap::IndexMap;
use insim::net::tokio_impl::Framed;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::{
    App,
    event::{Command, Dispatch, Shutdown, Startup},
    extract::{ExtractCx, FromContext},
    handler::ErasedHandler,
};
use crate::error::AppError;

/// Back-channel handle to the runtime.
///
/// Cloneable, cheap, and unbounded - sends never block (we trade backpressure
/// for freedom from a deadlock window where the dispatch task is itself the
/// only thing that can drain the channel). Two operations:
///
/// - [`Sender::packet`] - push a wire packet out to LFS.
/// - [`Sender::event`]  - inject a synthetic event into a new dispatch cycle.
///
/// Both routes end up at the same receiver in the dispatcher's main loop.
/// **Emission semantics: regardless of caller (handler, spawned task,
/// anywhere), events posted here are processed in a *subsequent* dispatch
/// cycle - not the current one.** This is the only emission API in the crate.
#[derive(Clone)]
pub struct Sender {
    tx: mpsc::UnboundedSender<Command>,
}

impl std::fmt::Debug for Sender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sender").finish_non_exhaustive()
    }
}

impl Sender {
    pub(crate) fn new(tx: mpsc::UnboundedSender<Command>) -> Self {
        Self { tx }
    }

    /// Send a single packet back to LFS. Non-blocking; only errors if the
    /// runtime has shut down (back-channel closed).
    pub fn packet<P: Into<insim::Packet>>(&self, packet: P) -> Result<(), AppError> {
        self.tx
            .send(Command::Packet(packet.into()))
            .map_err(|_| AppError::Closed)
    }

    /// Send one or more packets back to LFS. Non-blocking; only errors if the
    /// runtime has shut down (back-channel closed).
    pub fn packets<I, P>(&self, packets: I) -> Result<(), AppError>
    where
        I: IntoIterator<Item = P>,
        P: Into<insim::Packet>,
    {
        for p in packets {
            self.tx
                .send(Command::Packet(p.into()))
                .map_err(|_| AppError::Closed)?;
        }
        Ok(())
    }

    /// Inject a synthetic event into a new dispatch cycle. Non-blocking; only
    /// errors if the runtime has shut down.
    pub fn event<E: Any + Send + Sync + 'static>(&self, event: E) -> Result<(), AppError> {
        self.tx
            .send(Command::Event(std::sync::Arc::new(event)))
            .map_err(|_| AppError::Closed)
    }
}

impl<S> FromContext<S> for Sender {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        Some(cx.sender.clone())
    }
}

/// Connect to LFS using `builder` and run the dispatch loop until the
/// connection drops or the back-channel closes.
///
/// The builder carries protocol/ISI configuration (flags, prefix, iname, etc.);
/// `connect_async` is called once.
///
/// Immediately after the connection is established the runtime emits a
/// [`Startup`] synthetic event so handlers can install background work.
pub async fn run<S>(builder: insim::builder::Builder, app: App<S>) -> Result<(), AppError>
where
    S: Send + Sync + 'static,
{
    let state = app.state;
    let pre_handlers = app.pre_handlers;
    let update_handlers = app.update_handlers;
    let sender = app.sender;
    let cancel = app.cancel;
    let mut cmd_rx = app
        .cmd_rx
        .expect("App::cmd_rx already taken - run() must be called at most once");

    let mut framed = builder.connect_async().await?;

    // One-shot lifecycle event so handlers can `tokio::spawn` background work.
    dispatch_cycle(
        Dispatch::Synthetic(Arc::new(Startup)),
        &sender,
        &state,
        &pre_handlers,
        &update_handlers,
        &cancel,
    )
    .await;

    let result = run_dispatch_loop(
        &mut framed,
        &sender,
        &state,
        &pre_handlers,
        &update_handlers,
        &mut cmd_rx,
        &cancel,
    )
    .await;

    // Cancel before Shutdown so handlers can observe the token is fired.
    cancel.cancel();

    // One-shot lifecycle event so handlers can do cleanup and send final packets.
    dispatch_cycle(
        Dispatch::Synthetic(Arc::new(Shutdown)),
        &sender,
        &state,
        &pre_handlers,
        &update_handlers,
        &cancel,
    )
    .await;

    // Drain any packets queued during the Shutdown cycle. The connection may
    // already be gone so write errors are ignored.
    while let Ok(cmd) = cmd_rx.try_recv() {
        if let Command::Packet(p) = cmd {
            let _ = framed.write(p).await;
        }
    }

    result
}

async fn run_dispatch_loop<S>(
    framed: &mut Framed,
    sender: &Sender,
    state: &S,
    pre_handlers: &IndexMap<TypeId, Box<dyn ErasedHandler<S>>>,
    update_handlers: &IndexMap<TypeId, Box<dyn ErasedHandler<S>>>,
    cmd_rx: &mut mpsc::UnboundedReceiver<Command>,
    cancel: &CancellationToken,
) -> Result<(), AppError>
where
    S: Send + Sync + 'static,
{
    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => return Ok(()),
            res = framed.read() => {
                let packet = res?;
                dispatch_cycle(
                    Dispatch::Packet(packet),
                    sender, state, pre_handlers, update_handlers, cancel,
                ).await;
            }
            maybe_cmd = cmd_rx.recv() => {
                let Some(cmd) = maybe_cmd else { return Ok(()) };
                match cmd {
                    Command::Packet(packet) => {
                        if let Err(e) = framed.write(packet).await {
                            tracing::error!(?e, "write failed");
                        }
                    }
                    Command::Event(payload) => {
                        dispatch_cycle(
                            Dispatch::Synthetic(payload),
                            sender, state, pre_handlers, update_handlers, cancel,
                        ).await;
                    }
                }
            }
        }
    }
}

/// Drive one dispatch cycle for one event, in two phases.
///
/// 1. **Pre** - each handler is awaited *sequentially* in registration
///    order. State-mirror handlers (Presence, Game) live here so subsequent
///    handlers observe a settled world.
/// 2. **Update** - handlers run *concurrently* via [`FuturesUnordered`].
///    Most game logic lives here.
///
/// Synthetic events injected via `sender.event(...)` are *not* drained in
/// this cycle. They land on the runtime's back-channel and trigger their
/// own future cycles.
pub(crate) async fn dispatch_cycle<S>(
    d: Dispatch,
    sender: &Sender,
    state: &S,
    pre_handlers: &IndexMap<TypeId, Box<dyn ErasedHandler<S>>>,
    update_handlers: &IndexMap<TypeId, Box<dyn ErasedHandler<S>>>,
    cancel: &CancellationToken,
) where
    S: Send + Sync + 'static,
{
    let xcx = ExtractCx {
        dispatch: &d,
        sender,
        pre_handlers,
        update_handlers,
        cancel,
        state,
    };

    for h in pre_handlers.values() {
        if let Err(e) = h.call(&xcx).await {
            tracing::error!(?e, "pre handler failed");
        }
    }

    let mut pending: FuturesUnordered<_> = update_handlers.values().map(|h| h.call(&xcx)).collect();
    while let Some(result) = pending.next().await {
        if let Err(e) = result {
            tracing::error!(?e, "update handler failed");
        }
    }
}
