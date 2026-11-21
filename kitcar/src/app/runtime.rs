//! Runtime: the [`run`] and [`run_connection`] entry points that drive an
//! [`crate::App`] against a live connection, and the [`Sender`] back-channel
//! that hands packets/events to the runtime from anywhere.
//!
//! [`run`] is a deprecated convenience wrapper that opens a connection from an
//! `insim::builder::Builder`. [`run_connection`] accepts an already-established
//! connection, emits a one-shot [`crate::Startup`] synthetic event, then runs
//! the dispatch loop until the connection drops or the back-channel closes.

use std::{any::Any, sync::Arc};

use insim::net::{self, Response, Session, Tokio};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use super::{
    App,
    event::{Command, Dispatch, Shutdown, Startup},
    extract::{ExtractCx, FromContext},
    handler::ErasedHandler,
};
use crate::{World, WorldReady, error::AppError};

/// Bound on the connection's outgoing command queue and on packets waiting for
/// the dispatcher. Senders wait when it is full.
const CAPACITY: usize = 256;

/// The connection a [`Sender`] writes to.
#[derive(Debug, Clone)]
pub(crate) enum Link {
    /// The app has not been run yet.
    Pending,
    /// The app is running against this connection.
    Bound(net::Sender),
    /// The connection has ended.
    Closed,
}

/// Back-channel handle to the runtime.
///
/// Cloneable and cheap. Three kinds of operation:
///
/// - [`Sender::packet`] / [`Sender::packets`] - send wire packets to LFS. These
///   wait for room in the connection's bounded queue, so a burst (such as a UI
///   re-render for many connections) is paced rather than buffered without limit.
/// - [`Sender::request`] / [`Sender::request_all`] - send a request and wait for
///   the replies that echo it. The replies are still dispatched as packets.
/// - [`Sender::event`] - inject a synthetic event into a new dispatch cycle.
///   Never waits.
///
/// A `Sender` exists before the app is run, so handler constructors and
/// background tasks can hold one. Packets sent before [`run_connection`] starts
/// wait until it does; after the connection ends they fail with
/// [`AppError::Closed`].
///
/// **Emission semantics: regardless of caller (handler, spawned task,
/// anywhere), events posted here are processed in a *subsequent* dispatch
/// cycle - not the current one.**
#[derive(Clone)]
pub struct Sender {
    link: watch::Receiver<Link>,
    events: mpsc::UnboundedSender<Command>,
}

impl std::fmt::Debug for Sender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sender").finish_non_exhaustive()
    }
}

fn closed(error: insim::Error) -> AppError {
    match error {
        insim::Error::Disconnected => AppError::Closed,
        error => AppError::Insim(error),
    }
}

impl Sender {
    /// A sender with no connection: packets fail with [`AppError::Closed`].
    #[cfg(test)]
    pub(crate) fn new(events: mpsc::UnboundedSender<Command>) -> Self {
        let (_, link) = watch::channel(Link::Closed);
        Self { link, events }
    }

    /// A sender whose connection is bound later through the returned handle.
    pub(crate) fn pending(events: mpsc::UnboundedSender<Command>) -> (Self, watch::Sender<Link>) {
        let (bind, link) = watch::channel(Link::Pending);
        (Self { link, events }, bind)
    }

    async fn connection(&self) -> Result<net::Sender, AppError> {
        let mut link = self.link.clone();
        let link = link
            .wait_for(|link| !matches!(link, Link::Pending))
            .await
            .map_err(|_| AppError::Closed)?;
        match &*link {
            Link::Bound(sender) => Ok(sender.clone()),
            Link::Pending | Link::Closed => Err(AppError::Closed),
        }
    }

    /// Send a single packet to LFS, waiting for room in the outgoing queue.
    pub async fn packet<P: Into<insim::Packet>>(&self, packet: P) -> Result<(), AppError> {
        self.connection().await?.send(packet).await.map_err(closed)
    }

    /// Send packets to LFS in order, waiting for room in the outgoing queue.
    pub async fn packets<I, P>(&self, packets: I) -> Result<(), AppError>
    where
        I: IntoIterator<Item = P>,
        P: Into<insim::Packet>,
    {
        let connection = self.connection().await?;
        for packet in packets {
            connection.send(packet).await.map_err(closed)?;
        }
        Ok(())
    }

    /// Send a request and wait for the first reply of type `T`. See
    /// [`insim::net::Sender::request`].
    pub async fn request<T: Response>(
        &self,
        packet: impl Into<insim::Packet>,
    ) -> Result<T, AppError> {
        self.connection()
            .await?
            .request(packet)
            .await
            .map_err(closed)
    }

    /// Send a request and collect every reply of type `T` until they go quiet.
    /// See [`insim::net::Sender::request_all`].
    pub async fn request_all<T: Response>(
        &self,
        packet: impl Into<insim::Packet>,
    ) -> Result<Vec<T>, AppError> {
        self.connection()
            .await?
            .request_all(packet)
            .await
            .map_err(closed)
    }

    /// Inject a synthetic event into a new dispatch cycle. Non-blocking; only
    /// errors if the runtime has shut down.
    pub fn event<E: Any + Send + Sync + 'static>(&self, event: E) -> Result<(), AppError> {
        self.events
            .send(Command::Event(Arc::new(event)))
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
#[deprecated(
    note = "Create and handshake an insim::Session, then call run_connection(session, app)"
)]
#[allow(deprecated)]
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
/// One call represents one connection epoch. The session is split: a driver task
/// owns it and keeps reading (so keepalives are answered even while handlers are
/// slow), and the app's [`Sender`] is bound to it. The app's world is synchronized
/// with LFS, [`Startup`] is emitted once, [`WorldReady`] follows once every reply
/// to the startup requests has been dispatched, and [`Shutdown`] is emitted before
/// returning. Packets sent during the `Shutdown` cycle are flushed, followed by
/// `TINY_CLOSE`.
///
/// Callers that supervise or reconnect should construct a fresh [`App`] for each
/// connection and call this function again.
pub async fn run_connection<S, V>(
    connection: Session<Tokio>,
    app: App<S, V>,
) -> Result<(), AppError>
where
    S: Send + Sync + 'static,
    V: crate::ui::View + 'static,
{
    let state = app.state;
    let world = app.world;
    let ui_global = app.ui;
    let mut handlers = app.handlers;
    let sender = app.sender;
    let cancel = app.cancel;
    let bind = app.link;
    let mut events = app
        .events
        .expect("App::events already taken - an App can only be run once");

    let (connection, mut receiver, driver) = connection.into_split(CAPACITY);
    let driver = tokio::spawn(driver);
    let _ = bind.send_replace(Link::Bound(connection.clone()));

    // Attach before requesting startup state, so the UI sees those replies too.
    let ui = match ui_global {
        Some(global) => crate::ui::Ui::attach(&connection, sender.clone(), global).await?,
        None => crate::ui::Ui::disabled(),
    };

    // Sync the world with the server's current state (LFS does not volunteer it).
    let startup = {
        let connection = connection.clone();
        tokio::spawn(async move {
            World::request_startup(&connection)
                .await
                .map_err(AppError::from)
        })
    };

    // One-shot lifecycle event so handlers can `tokio::spawn` background work.
    dispatch_cycle(
        Dispatch::Synthetic(Arc::new(Startup)),
        &sender,
        &world,
        &ui,
        &state,
        &mut handlers,
        &cancel,
    )
    .await;

    let result = run_dispatch_loop(
        &mut receiver,
        Ready::new(startup),
        &sender,
        &world,
        &ui,
        &state,
        &mut handlers,
        &mut events,
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
        &mut handlers,
        &cancel,
    )
    .await;

    // Everything accepted so far goes out before TINY_CLOSE. The connection may
    // already be gone, so errors are ignored.
    let _ = connection.shutdown().await;
    let _ = bind.send_replace(Link::Closed);
    drop((connection, receiver));
    match driver.await {
        Ok(Err(error)) => tracing::debug!(%error, "connection driver ended with an error"),
        Ok(Ok(())) => {},
        Err(error) => tracing::error!(%error, "connection driver panicked"),
    }

    result
}

/// Tracks the startup requests so [`WorldReady`] is dispatched only after every
/// reply to them has been folded into the world.
///
/// A reply future resolves when its packets are *decoded*, which can be before
/// the dispatcher has processed them. So the dispatcher counts the packets it
/// dispatches for each ReqI and compares them with the replies once known.
struct Ready {
    task: Option<JoinHandle<Result<Vec<insim::Packet>, AppError>>>,
    seen: [usize; 256],
    expected: Option<[usize; 256]>,
}
impl Ready {
    fn new(task: JoinHandle<Result<Vec<insim::Packet>, AppError>>) -> Self {
        Self {
            task: Some(task),
            seen: [0; 256],
            expected: None,
        }
    }
    fn dispatched(&mut self, packet: &insim::Packet) {
        if let Some(reqi) = packet.request_id() {
            self.seen[usize::from(reqi.0)] += 1;
        }
    }
    async fn replies(&mut self) -> Option<Vec<insim::Packet>> {
        let task = self.task.as_mut()?;
        let result = task.await;
        self.task = None;
        match result {
            Ok(Ok(replies)) => Some(replies),
            Ok(Err(error)) => {
                tracing::warn!(%error, "startup requests failed; WorldReady will not be emitted");
                None
            },
            Err(error) => {
                tracing::error!(%error, "startup request task panicked");
                None
            },
        }
    }
    fn expect(&mut self, replies: Vec<insim::Packet>) {
        let mut expected = [0; 256];
        for reqi in replies.iter().filter_map(insim::Packet::request_id) {
            expected[usize::from(reqi.0)] += 1;
        }
        self.expected = Some(expected);
    }
    /// True exactly once, when every startup reply has been dispatched.
    fn take(&mut self) -> bool {
        let done = self
            .expected
            .is_some_and(|expected| expected.iter().zip(&self.seen).all(|(e, s)| s >= e));
        if done {
            self.expected = None;
        }
        done
    }
}

#[allow(clippy::too_many_arguments)] // runtime plumbing; bundling would re-churn the dispatch_cycle call sites
async fn run_dispatch_loop<S, V>(
    receiver: &mut net::Receiver,
    mut ready: Ready,
    sender: &Sender,
    world: &World,
    ui: &crate::ui::Ui<V>,
    state: &S,
    handlers: &mut [Box<dyn ErasedHandler<S, V>>],
    events: &mut mpsc::UnboundedReceiver<Command>,
    cancel: &CancellationToken,
) -> Result<(), AppError>
where
    S: Send + Sync + 'static,
    V: crate::ui::View + 'static,
{
    loop {
        let waiting = ready.task.is_some();
        tokio::select! {
            biased;
            _ = cancel.cancelled() => return Ok(()),
            item = receiver.recv() => match item {
                None => return Ok(()),
                Some(Err(error @ insim::Error::IncompleteDecode { .. })) => {
                    tracing::warn!(%error, "skipping a partially decoded packet");
                },
                Some(Err(error)) => return Err(error.into()),
                Some(Ok(packet)) => {
                    ready.dispatched(&packet);
                    dispatch_cycle(
                        Dispatch::Packet(packet),
                        sender, world, ui, state, handlers, cancel,
                    ).await;
                },
            },
            replies = ready.replies(), if waiting => {
                if let Some(replies) = replies {
                    ready.expect(replies);
                }
            },
            command = events.recv() => {
                let Some(Command::Event(payload)) = command else { return Ok(()) };
                dispatch_cycle(
                    Dispatch::Synthetic(payload),
                    sender, world, ui, state, handlers, cancel,
                ).await;
            },
        }
        if ready.take() {
            dispatch_cycle(
                Dispatch::Synthetic(Arc::new(WorldReady::default())),
                sender,
                world,
                ui,
                state,
                handlers,
                cancel,
            )
            .await;
        }
    }
}

/// Drive one dispatch for one event.
///
/// A wire packet is first folded into the intrinsic [`World`] mirror (via
/// [`fold_packet`]), so every handler observes settled state. The packet is
/// then run through the handlers, and finally each world event the fold
/// produced is run through them too - in this same call, right after the
/// causing packet, so there is **no inter-cycle delay** for world events.
///
/// Handlers are awaited sequentially in registration order.
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
    handlers: &mut [Box<dyn ErasedHandler<S, V>>],
    cancel: &CancellationToken,
) where
    S: Send + Sync + 'static,
    V: crate::ui::View + 'static,
{
    let derived = if let Dispatch::Packet(packet) = &d {
        crate::world::fold_packet(world, packet, sender).await
    } else {
        Vec::new()
    };

    run_handlers(&d, sender, world, ui, state, handlers, cancel).await;

    for event in derived {
        run_handlers(&event, sender, world, ui, state, handlers, cancel).await;
    }
}

/// Run one dispatch sequentially through the registered handlers.
#[allow(clippy::too_many_arguments)] // runtime plumbing; threads world + ui through each cycle
async fn run_handlers<S, V>(
    d: &Dispatch,
    sender: &Sender,
    world: &World,
    ui: &crate::ui::Ui<V>,
    state: &S,
    handlers: &mut [Box<dyn ErasedHandler<S, V>>],
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
        cancel,
        state,
    };

    for h in handlers.iter_mut() {
        if let Err(e) = h.call(&xcx).await {
            tracing::error!(?e, "handler failed");
        }
    }
}
