//! [`App`] - the composition root - and [`serve`] - the runner.
//!
//! [`App`] holds handlers, middleware, and shared state; it does not itself
//! open a connection. [`serve`] consumes an [`App`] together with an
//! `insim::Builder`, opens the connection, emits a one-shot [`crate::Startup`]
//! synthetic event, and drives the single-task dispatch loop.

use std::{ops::Deref, sync::Arc};

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
    middleware::{ErasedExtension, EventCx, Extension},
};

// ---------------------------------------------------------------------------
// App<S> - composition root
// ---------------------------------------------------------------------------

/// Composition root for an `insim_app` bot.
///
/// Build up state, handlers, and middleware, then hand the value to [`serve`].
///
/// Long-running background tasks (periodic tickers, polls, etc.) are *not* a
/// special primitive here: register a handler on [`Event<Startup>`] and call
/// `tokio::spawn` from inside it.
///
/// [`Event<Startup>`]: crate::Event
pub struct App<S> {
    pub(crate) state: Option<S>,
    pub(crate) handlers: Vec<Box<dyn ErasedHandler<S>>>,
    pub(crate) extension_chain: Vec<Arc<dyn ErasedExtension<S>>>,
    pub(crate) extensions: Extensions,
    pub(crate) sender: Sender,
    /// Receiver paired with `sender`. Taken by `serve()`.
    pub(crate) cmd_rx: Option<mpsc::UnboundedReceiver<Command>>,
    /// Runtime cancellation. Cloned into [`ExtractCx`] / [`EventCx`] so handlers
    /// and extensions can call `cx.shutdown()` to bring `serve` down.
    pub(crate) cancel: CancellationToken,
}

impl<S> std::fmt::Debug for App<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("handlers", &self.handlers.len())
            .field("extension_chain", &self.extension_chain.len())
            .field("extensions", &self.extensions)
            .field("has_state", &self.state.is_some())
            .finish()
    }
}

impl<S> Default for App<S> {
    fn default() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<Command>();
        Self {
            state: None,
            handlers: Vec::new(),
            extension_chain: Vec::new(),
            extensions: Extensions::new(),
            sender: Sender::new(cmd_tx),
            cmd_rx: Some(cmd_rx),
            cancel: CancellationToken::new(),
        }
    }
}

impl<S> App<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Create an empty app.
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrow the runtime's outbound `Sender`. Cloneable; lives for the
    /// lifetime of the `App` and remains valid through `serve()`. Useful
    /// for extensions (like the UI subsystem) that need a `Sender` at
    /// construction time rather than picking one up in `on_event`.
    pub fn sender(&self) -> &Sender {
        &self.sender
    }

    /// Borrow the runtime's [`CancellationToken`]. Cloneable; cancelling it
    /// (or any clone) requests the dispatcher exit at its next select.
    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel
    }

    /// Set the shared application state. Required before [`serve`] will run.
    #[must_use]
    pub fn with_state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    /// Register a handler. The handler's parameter types determine which
    /// dispatches it runs on - see [`crate::Packet`] / [`crate::Event`].
    #[must_use]
    pub fn handler<T, H>(mut self, handler: H) -> Self
    where
        H: Handler<S, T> + 'static,
        T: Send + 'static,
    {
        self.handlers.push(Box::new(HandlerService::new(handler)));
        self
    }

    /// Register an extension. The value is:
    ///
    /// - **stored in the extension registry by its `TypeId`** so handlers can
    ///   pull it out via [`FromContext`](crate::FromContext), and
    /// - **added to the dispatch chain** so its `on_event` (default no-op) is
    ///   called for every dispatch, before handlers.
    ///
    /// Registration order = `on_event` order. The value is wrapped in `Arc`
    /// once and shared between both views - no `E: Clone` bound, no per-call
    /// clone of `E`.
    #[must_use]
    pub fn extension<E: Extension<S>>(mut self, value: E) -> Self {
        let arc: Arc<E> = Arc::new(value);
        self.extensions.insert_arc(arc.clone());
        self.extension_chain.push(arc);
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
/// `connect_async` is called once. `App::with_state` must have been called.
///
/// Immediately after the connection is established the runtime emits a
/// [`Startup`] synthetic event so handlers can install background work.
pub async fn serve<S>(builder: insim::builder::Builder, app: App<S>) -> Result<(), AppError>
where
    S: Clone + Send + Sync + 'static,
{
    let state = app
        .state
        .expect("App missing state; call .with_state(...) before serve");
    let extension_chain = app.extension_chain;
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
        &state,
        &sender,
        &extension_chain,
        &handlers,
        &extensions,
        &cancel,
    )
    .await;

    let result = run_dispatch_loop(
        &mut framed,
        &state,
        &sender,
        &extension_chain,
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

#[allow(clippy::too_many_arguments)]
async fn run_dispatch_loop<S>(
    framed: &mut Framed,
    state: &S,
    sender: &Sender,
    extension_chain: &[Arc<dyn ErasedExtension<S>>],
    handlers: &[Box<dyn ErasedHandler<S>>],
    extensions: &Extensions,
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
                    state, sender,
                    extension_chain, handlers, extensions, cancel,
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
                            state, sender,
                            extension_chain, handlers, extensions, cancel,
                        ).await;
                    }
                }
            }
        }
    }
}

/// Drive one dispatch cycle for one event.
///
/// Extensions' `on_event` runs first, in registration order, sequentially.
/// Handlers then run **concurrently** via [`FuturesUnordered`] - they only
/// observe `&state` and emit through the shared `Sender`, so any shared state
/// they touch must be atomic / lock-tolerant (`Arc<AtomicX>`, `Arc<RwLock<…>>`).
///
/// Synthetic events injected via `sender.event(...)` are *not* drained in this
/// cycle. They land on the runtime's back-channel and trigger their own
/// future cycles. This is the single emission semantic across extensions,
/// handlers, and spawned tasks.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn dispatch_cycle<S>(
    d: Dispatch,
    state: &S,
    sender: &Sender,
    extension_chain: &[Arc<dyn ErasedExtension<S>>],
    handlers: &[Box<dyn ErasedHandler<S>>],
    extensions: &Extensions,
    cancel: &CancellationToken,
) where
    S: Send + Sync + 'static,
{
    // Extensions in registration order, sequential.
    for ext in extension_chain.iter() {
        let mut evcx = EventCx {
            dispatch: &d,
            state,
            sender,
            extensions,
            cancel,
        };
        ext.deref().on_event(&mut evcx).await;
    }

    // Handlers concurrently.
    let xcx = ExtractCx {
        dispatch: &d,
        state,
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
