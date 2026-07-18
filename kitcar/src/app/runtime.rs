//! Runtime: the [`run`] and [`run_connection`] entry points that drive an
//! [`crate::App`] against a live connection, and the [`Sender`] back-channel
//! that hands packets/events to the runtime from anywhere.
//!
//! [`run`] is a convenience wrapper that opens a connection from an
//! `insim::Builder`. [`run_connection`] accepts an already-established
//! connection, emits a one-shot [`crate::Startup`] synthetic event, then runs
//! the dispatch loop until the connection drops or the back-channel closes.

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
use crate::{World, error::AppError};

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

impl<S, V: crate::ui::View + 'static> FromContext<S, V> for Sender {
    fn from_context(cx: &ExtractCx<'_, S, V>) -> Option<Self> {
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
pub async fn run<S, V>(builder: insim::builder::Builder, app: App<S, V>) -> Result<(), AppError>
where
    S: Send + Sync + 'static,
    V: crate::ui::View + 'static,
{
    let connection = builder.connect_async().await?;
    run_connection(connection, app).await
}

/// Run `app` against an established InSim connection until the connection
/// drops or the app shuts down.
///
/// One call represents one connection epoch. The app's world is synchronized
/// with LFS, [`Startup`] is emitted once, and [`Shutdown`] is emitted before
/// returning. Callers that supervise or reconnect should construct a fresh
/// [`App`] for each connection and call this function again.
pub async fn run_connection<S, V>(mut connection: Framed, app: App<S, V>) -> Result<(), AppError>
where
    S: Send + Sync + 'static,
    V: crate::ui::View + 'static,
{
    let state = app.state;
    let world = app.world;
    let ui = app.ui;
    let pre_handlers = app.pre_handlers;
    let update_handlers = app.update_handlers;
    let sender = app.sender;
    let cancel = app.cancel;
    let mut cmd_rx = app
        .cmd_rx
        .expect("App::cmd_rx already taken - an App can only be run once");

    // Sync the world with the server's current state (LFS does not volunteer it).
    crate::world::send_startup_requests(&sender);

    // One-shot lifecycle event so handlers can `tokio::spawn` background work.
    dispatch_cycle(
        Dispatch::Synthetic(Arc::new(Startup)),
        &sender,
        &world,
        &ui,
        &state,
        &pre_handlers,
        &update_handlers,
        &cancel,
    )
    .await;

    let result = run_dispatch_loop(
        &mut connection,
        &sender,
        &world,
        &ui,
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
        &world,
        &ui,
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
            let _ = connection.write(p).await;
        }
    }

    result
}

#[allow(clippy::too_many_arguments)] // runtime plumbing; bundling would re-churn the dispatch_cycle call sites
async fn run_dispatch_loop<S, V>(
    framed: &mut Framed,
    sender: &Sender,
    world: &World,
    ui: &crate::ui::Ui<V>,
    state: &S,
    pre_handlers: &IndexMap<TypeId, Box<dyn ErasedHandler<S, V>>>,
    update_handlers: &IndexMap<TypeId, Box<dyn ErasedHandler<S, V>>>,
    cmd_rx: &mut mpsc::UnboundedReceiver<Command>,
    cancel: &CancellationToken,
) -> Result<(), AppError>
where
    S: Send + Sync + 'static,
    V: crate::ui::View + 'static,
{
    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => return Ok(()),
            res = framed.read() => {
                let packet = res?;
                dispatch_cycle(
                    Dispatch::Packet(packet),
                    sender, world, ui, state, pre_handlers, update_handlers, cancel,
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
                            sender, world, ui, state, pre_handlers, update_handlers, cancel,
                        ).await;
                    }
                }
            }
        }
    }
}

/// Drive one dispatch for one event.
///
/// A wire packet is first folded into the intrinsic [`World`] mirror (via
/// [`fold_packet`]), so every handler observes settled state. The packet is
/// then run through the two handler phases, and finally each world event the
/// fold produced is run through them too - in this same call, right after the
/// causing packet, so there is **no inter-cycle delay** for world events.
///
/// The two phases (see [`run_handlers`]):
///
/// 1. **Pre** - handlers awaited *sequentially* in registration order.
/// 2. **Update** - handlers run *concurrently* via [`FuturesUnordered`].
///
/// Synthetic events injected by handlers via `sender.event(...)` are *not*
/// drained here. They land on the runtime's back-channel and trigger their own
/// future cycles.
#[allow(clippy::too_many_arguments)] // runtime plumbing; threads world + ui through each cycle
pub(crate) async fn dispatch_cycle<S, V>(
    d: Dispatch,
    sender: &Sender,
    world: &World,
    ui: &crate::ui::Ui<V>,
    state: &S,
    pre_handlers: &IndexMap<TypeId, Box<dyn ErasedHandler<S, V>>>,
    update_handlers: &IndexMap<TypeId, Box<dyn ErasedHandler<S, V>>>,
    cancel: &CancellationToken,
) where
    S: Send + Sync + 'static,
    V: crate::ui::View + 'static,
{
    let derived = if let Dispatch::Packet(packet) = &d {
        // Drive the UI before folding the world, so it sees every packet (the UI
        // runs on its own thread; this only enqueues, and is a no-op when inert).
        ui.forward_packet(packet);
        crate::world::fold_packet(world, packet, sender)
    } else {
        Vec::new()
    };

    run_handlers(
        &d,
        sender,
        world,
        ui,
        state,
        pre_handlers,
        update_handlers,
        cancel,
    )
    .await;

    for event in derived {
        run_handlers(
            &event,
            sender,
            world,
            ui,
            state,
            pre_handlers,
            update_handlers,
            cancel,
        )
        .await;
    }
}

/// Run one dispatch through the Pre (sequential) then Update (concurrent)
/// handler phases against a freshly built [`ExtractCx`].
#[allow(clippy::too_many_arguments)] // runtime plumbing; threads world + ui through each cycle
async fn run_handlers<S, V>(
    d: &Dispatch,
    sender: &Sender,
    world: &World,
    ui: &crate::ui::Ui<V>,
    state: &S,
    pre_handlers: &IndexMap<TypeId, Box<dyn ErasedHandler<S, V>>>,
    update_handlers: &IndexMap<TypeId, Box<dyn ErasedHandler<S, V>>>,
    cancel: &CancellationToken,
) where
    S: Send + Sync + 'static,
    V: crate::ui::View + 'static,
{
    let xcx = ExtractCx {
        dispatch: d,
        sender,
        world,
        ui,
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
