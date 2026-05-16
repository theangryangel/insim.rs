//! [`App`] - the composition root - and [`serve`] - the runner.
//!
//! [`App`] holds handlers and a typed resource registry; it does not itself
//! open a connection. [`serve`] consumes an [`App`] together with an
//! `insim::Builder`, opens the connection, emits a one-shot [`crate::Startup`]
//! synthetic event, and drives the single-task dispatch loop.

use std::{sync::Arc, time::Duration};

use futures::stream::{FuturesUnordered, StreamExt};
use insim::net::tokio_impl::Framed;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    error::AppError,
    event::{Command, Dispatch, Startup},
    extensions::Extensions,
    extract::{ExtractCx, Sender},
    handler::{ErasedHandler, Handler, HandlerService},
};

// ---------------------------------------------------------------------------
// Installable - bundles of resources + handlers
// ---------------------------------------------------------------------------

/// A bundle of related registrations that consume an [`App`] and return one.
///
/// Use this for types that pair a resource with the handlers that operate on
/// it — for example [`crate::Presence`] registers itself plus a dozen
/// packet-specific handlers. Implement on your own types to package
/// resource + handler ensembles for reuse:
///
/// ```ignore
/// impl Installable for MyMiniGame {
///     fn install(self, app: App) -> App {
///         app.resource(self)
///            .handler(my_minigame_on_uco)
///            .handler(my_minigame_on_tick)
///     }
/// }
/// ```
pub trait Installable {
    /// Consume the [`App`], add this bundle's resources/handlers, return it.
    fn install(self, app: App) -> App;
}

// ---------------------------------------------------------------------------
// App - composition root
// ---------------------------------------------------------------------------

/// Composition root for an `insim_app` bot.
///
/// Build up handlers and typed resources, then hand the value to [`serve`].
///
/// Periodic synthetic events have a small dedicated helper - see
/// [`App::periodic`]. Other long-running background tasks aren't a special
/// primitive: register a handler on [`Event<Startup>`] and `tokio::spawn`
/// from inside it, or use [`Spawned`](crate::Spawned) for tasks that also
/// need to observe every dispatch.
///
/// [`Event<Startup>`]: crate::Event
pub struct App {
    pub(crate) handlers: Vec<Box<dyn ErasedHandler>>,
    pub(crate) extensions: Extensions,
    pub(crate) sender: Sender,
    /// Receiver paired with `sender`. Taken by `serve()`.
    pub(crate) cmd_rx: Option<mpsc::UnboundedReceiver<Command>>,
    /// Runtime cancellation. Cloned into [`ExtractCx`] so handlers can call
    /// `cx.shutdown()` to bring `serve` down.
    pub(crate) cancel: CancellationToken,
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("handlers", &self.handlers.len())
            .field("extensions", &self.extensions)
            .finish()
    }
}

impl Default for App {
    fn default() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<Command>();
        Self {
            handlers: Vec::new(),
            extensions: Extensions::new(),
            sender: Sender::new(cmd_tx),
            cmd_rx: Some(cmd_rx),
            cancel: CancellationToken::new(),
        }
    }
}

impl App {
    /// Create an empty app.
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrow the runtime's outbound `Sender`. Cloneable; lives for the
    /// lifetime of the `App` and remains valid through `serve()`. Useful
    /// for resource constructors (like `Presence::new(sender)`) that need a
    /// `Sender` at construction time.
    pub fn sender(&self) -> &Sender {
        &self.sender
    }

    /// Borrow the runtime's [`CancellationToken`]. Cloneable; cancelling it
    /// (or any clone) requests the dispatcher exit at its next select.
    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel
    }

    /// Register a handler. The handler's parameter types determine which
    /// dispatches it runs on - see [`crate::Packet`] / [`crate::Event`].
    #[must_use]
    pub fn handler<T, H>(mut self, handler: H) -> Self
    where
        H: Handler<T> + 'static,
        T: Send + 'static,
    {
        self.handlers.push(Box::new(HandlerService::new(handler)));
        self
    }

    /// Register a typed resource. The value is inserted into the registry
    /// keyed by its `TypeId` so handlers can pull it out via [`FromContext`]
    /// (the resource author implements [`FromContext`] on the type itself)
    /// or via the [`crate::Res`] wrapper extractor.
    ///
    /// Resources are pure typed data - they don't observe dispatches. To
    /// react to dispatches, register handlers; to bundle a resource together
    /// with its handlers, use [`App::install`].
    ///
    /// [`FromContext`]: crate::FromContext
    #[must_use]
    pub fn resource<R: Send + Sync + 'static>(mut self, value: R) -> Self {
        self.extensions.insert_arc(Arc::new(value));
        self
    }

    /// Install an [`Installable`] bundle - typically a resource plus a fixed
    /// set of handlers that operate on it (e.g. [`crate::Presence`]).
    /// Equivalent to `installable.install(self)`.
    #[must_use]
    pub fn install<I: Installable>(self, installable: I) -> Self {
        installable.install(self)
    }

    /// Spawn a background task that emits `event` as a synthetic event every
    /// `period`, until the runtime's cancel token fires.
    ///
    /// Use this for fire-and-forget periodic emitters where the event payload
    /// is static or cheaply cloneable (a unit struct, a counter, etc.). For
    /// anything more complex - per-tick state, side effects beyond the event,
    /// custom shutdown handling - reach for [`crate::Spawned`].
    ///
    /// The interval uses [`tokio::time::MissedTickBehavior::Skip`]: if the
    /// runtime pauses, missed ticks are dropped rather than burst-fired.
    ///
    /// The task is spawned immediately, so `App::periodic` **must be called
    /// inside a tokio runtime context**. Events queue on the runtime's
    /// back-channel and are drained once [`serve`] begins.
    #[must_use]
    pub fn periodic<E>(self, period: Duration, event: E) -> Self
    where
        E: std::any::Any + Clone + Send + Sync + 'static,
    {
        let sender = self.sender.clone();
        let cancel = self.cancel.clone();
        drop(tokio::spawn(async move {
            let mut tick = tokio::time::interval(period);
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => return,
                    _ = tick.tick() => {
                        if sender.event(event.clone()).is_err() {
                            return;
                        }
                    }
                }
            }
        }));
        self
    }
}

// ---------------------------------------------------------------------------
// serve - runs the App
// ---------------------------------------------------------------------------

/// Connect to LFS using `builder` and run the dispatch loop until the
/// connection drops or the back-channel closes.
///
/// The builder carries protocol/ISI configuration (flags, prefix, iname, etc.);
/// `connect_async` is called once.
///
/// Immediately after the connection is established the runtime emits a
/// [`Startup`] synthetic event so handlers can install background work.
pub async fn serve(builder: insim::builder::Builder, app: App) -> Result<(), AppError> {
    let handlers = app.handlers;
    let extensions = app.extensions;
    let sender = app.sender;
    let cancel = app.cancel;
    let mut cmd_rx = app
        .cmd_rx
        .expect("App::cmd_rx already taken - serve() must be called at most once");

    let mut framed = builder.connect_async().await?;

    // One-shot lifecycle event so handlers can `tokio::spawn` background work.
    dispatch_cycle(
        Dispatch::Synthetic(Arc::new(Startup)),
        &sender,
        &handlers,
        &extensions,
        &cancel,
    )
    .await;

    let result = run_dispatch_loop(
        &mut framed,
        &sender,
        &handlers,
        &extensions,
        &mut cmd_rx,
        &cancel,
    )
    .await;

    // Cooperatively signal anyone listening on the token (UI thread, etc.).
    cancel.cancel();
    result
}

async fn run_dispatch_loop(
    framed: &mut Framed,
    sender: &Sender,
    handlers: &[Box<dyn ErasedHandler>],
    extensions: &Extensions,
    cmd_rx: &mut mpsc::UnboundedReceiver<Command>,
    cancel: &CancellationToken,
) -> Result<(), AppError> {
    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => return Ok(()),
            res = framed.read() => {
                let packet = res?;
                dispatch_cycle(
                    Dispatch::Packet(packet),
                    sender, handlers, extensions, cancel,
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
                            sender, handlers, extensions, cancel,
                        ).await;
                    }
                }
            }
        }
    }
}

/// Drive one dispatch cycle for one event.
///
/// All registered handlers are tried concurrently via [`FuturesUnordered`] -
/// each one's extractors filter it in or out, and those that pass run their
/// bodies in parallel. They observe shared resources via `&Extensions` and
/// emit through the shared `Sender`, so any state two handlers may mutate
/// concurrently must be atomic / lock-tolerant (`Arc<AtomicX>`,
/// `Arc<RwLock<…>>`).
///
/// Synthetic events injected via `sender.event(...)` are *not* drained in this
/// cycle. They land on the runtime's back-channel and trigger their own
/// future cycles. This is the single emission semantic across handlers and
/// spawned tasks.
pub(crate) async fn dispatch_cycle(
    d: Dispatch,
    sender: &Sender,
    handlers: &[Box<dyn ErasedHandler>],
    extensions: &Extensions,
    cancel: &CancellationToken,
) {
    let xcx = ExtractCx {
        dispatch: &d,
        sender,
        extensions,
        cancel,
    };
    let mut pending: FuturesUnordered<_> = handlers.iter().map(|h| h.call(&xcx)).collect();
    while let Some(result) = pending.next().await {
        if let Err(e) = result {
            tracing::error!(?e, "handler failed");
        }
    }
}
