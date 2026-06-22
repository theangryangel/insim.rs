//! [`App`] - the composition root.
//!
//! [`App`] holds handlers (one IndexMap per stage); it does not itself open a
//! connection. Hand the value to [`crate::run`] when ready to run.

pub(crate) mod event;
pub(crate) mod extract;
pub(crate) mod handler;
pub mod run_if;
pub(crate) mod runtime;
#[cfg(test)]
mod tests;

use std::{any::TypeId, time::Duration};

use indexmap::IndexMap;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use self::{
    event::Command,
    handler::{ErasedHandler, Handler, HandlerService},
    runtime::Sender,
};
use crate::{
    World,
    ui::{NoView, View},
};

/// Which stage of the dispatch cycle a handler runs in.
///
/// - [`Stage::Pre`] - handlers run *sequentially* in registration order at
///   the start of each dispatch. Deciders that Update handlers gate on (e.g.
///   [`crate::RoundManager`]) live here so their effects settle first. The
///   intrinsic [`crate::World`] mirror is folded by the runtime ahead of both
///   stages.
/// - [`Stage::Update`] - handlers run *concurrently* after every Pre
///   handler has finished. Most game logic belongs here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stage {
    /// Sequential stage running first. Use for state mirrors and other
    /// handlers that must finish before any Update handler observes the
    /// same dispatch.
    Pre,
    /// Concurrent stage running after Pre. Most handlers belong here.
    Update,
}

/// A bundle of registrations that consume an [`App<S>`] and return one.
///
/// Useful when a plugin wants to add several handlers (and/or sub-handlers)
/// in one call. Most stateful handlers don't need this - they just impl
/// [`Handler`] directly and the user registers them with
/// [`App::handle`].
///
/// ```ignore
/// struct MyChatCommands;
///
/// impl<S: Send + Sync + 'static> Installable<S> for MyChatCommands {
///     fn install(self, app: App<S>) -> App<S> {
///         app.handle(Stage::Update, on_help)
///            .handle(Stage::Update, on_ping)
///     }
/// }
/// ```
pub trait Installable<S = (), V = NoView>
where
    V: View + 'static,
{
    /// Consume the [`App<S, V>`], add this bundle's registrations, return it.
    fn install(self, app: App<S, V>) -> App<S, V>;
}

/// Composition root for an `kitcar` bot.
///
/// Build up handlers, then hand the value to [`crate::run`].
///
/// Apps are parameterised by their primary state type `S`. Stateless bots use
/// `App<()>` via [`App::new`]; stateful bots use [`App::with_state`] to lock
/// in a state value that handlers extract via [`crate::State<S>`]. The
/// framework holds the state in an `Arc<RwLock<S>>` internally, so users
/// write plain `S` without any wrapping of their own.
///
/// ## Dispatch model
///
/// Every dispatch runs in two phases:
///
/// 1. **Pre** - [`Stage::Pre`] handlers run *sequentially* in registration
///    order. Deciders that Update handlers gate on (e.g. [`crate::RoundManager`])
///    belong here.
/// 2. **Update** - [`Stage::Update`] handlers run *concurrently* via
///    [`futures::stream::FuturesUnordered`]. Most game logic belongs here.
///
/// Every handler registered via [`App::handle`] is also inserted into a
/// per-stage `TypeId`-keyed map; that map serves both for dispatch (ordered
/// iteration) and for typed extraction (other handlers extract a stateful
/// handler value by its concrete type). There is no separate "registry"
/// data structure.
///
/// Periodic synthetic events have a small dedicated helper - see
/// [`App::periodic`].
pub struct App<S = (), V = NoView>
where
    V: View + 'static,
{
    pub(crate) state: S,
    /// The world-state mirror, intrinsic to every app (as core as the
    /// connection itself). Folded by the runtime's mirror step before any
    /// handler runs each cycle, and extractable via `world: World`.
    pub(crate) world: World,
    /// The app's UI, intrinsic like `world` and parameterising the app via `V`.
    /// A dedicated runtime-driven slot (not a handler): the runtime forwards
    /// packets to it each cycle before handlers run. Set via [`App::with_ui`];
    /// extracted infallibly as `ui: Ui<V>`. An app with no UI keeps the inert
    /// [`NoView`] handle.
    pub(crate) ui: crate::ui::Ui<V>,
    /// Pre-stage handlers, keyed by handler `TypeId`. IndexMap preserves
    /// insertion order for dispatch and supports O(1) lookup for extraction.
    pub(crate) pre_handlers: IndexMap<TypeId, Box<dyn ErasedHandler<S, V>>>,
    /// Update-stage handlers, ditto.
    pub(crate) update_handlers: IndexMap<TypeId, Box<dyn ErasedHandler<S, V>>>,
    pub(crate) sender: Sender,
    /// Receiver paired with `sender`. Taken by `run()`.
    pub(crate) cmd_rx: Option<mpsc::UnboundedReceiver<Command>>,
    /// Runtime cancellation. Cloned into [`ExtractCx`] so handlers can call
    /// `cx.shutdown()` to bring `run` down.
    pub(crate) cancel: CancellationToken,
}

impl<S, V: View + 'static> std::fmt::Debug for App<S, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("world", &self.world)
            .field("ui", &self.ui)
            .field("pre_handlers", &self.pre_handlers.len())
            .field("update_handlers", &self.update_handlers.len())
            .finish()
    }
}

impl Default for App<()> {
    fn default() -> Self {
        Self::with_state(())
    }
}

impl App<()> {
    /// Create an empty, stateless app (`App<()>`).
    ///
    /// For a stateful app, use [`App::with_state`] instead.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S> App<S, NoView>
where
    S: Clone + Send + Sync + 'static,
{
    /// Create an empty app whose primary state is `state`.
    ///
    /// `S` must be `Clone + Send + Sync + 'static`: handlers receive a
    /// clone via the [`crate::State<S>`] extractor. The framework does not
    /// wrap your state in any lock - if you need shared mutability, build
    /// it into `S` (`Arc<Mutex<…>>`, `Arc<RwLock<…>>`, `Arc<AtomicX>`, or
    /// plain `Arc<…>` for read-only data).
    ///
    /// The app starts with no UI ([`NoView`]); add one with [`App::with_ui`].
    pub fn with_state(state: S) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<Command>();
        Self {
            state,
            world: World::new(),
            ui: crate::ui::Ui::disabled(),
            pre_handlers: IndexMap::new(),
            update_handlers: IndexMap::new(),
            sender: Sender::new(cmd_tx),
            cmd_rx: Some(cmd_rx),
            cancel: CancellationToken::new(),
        }
    }

    /// Register the app's UI, fixing the view type `V` and switching the app to
    /// `App<S, V>`. Builds the [`Ui`](crate::ui::Ui) from the app's own sender;
    /// `initial_global` seeds the global state, and each connection's view is
    /// constructed via [`View::mount`].
    ///
    /// **Must be called before any handlers are registered** (it resets the
    /// handler maps to the new view type); a `debug_assert` guards against
    /// late calls. The UI then lives in a dedicated runtime-driven slot - the
    /// runtime forwards packets to it each cycle before handlers run, and
    /// handlers extract it infallibly as `ui: Ui<V>`.
    #[must_use]
    pub fn with_ui<V>(self, initial_global: V::Global) -> App<S, V>
    where
        V: View + 'static,
    {
        debug_assert!(
            self.pre_handlers.is_empty() && self.update_handlers.is_empty(),
            "App::with_ui must be called before any handlers are registered"
        );
        let ui = crate::ui::Ui::new(self.sender.clone(), initial_global);
        App {
            state: self.state,
            world: self.world,
            ui,
            pre_handlers: IndexMap::new(),
            update_handlers: IndexMap::new(),
            sender: self.sender,
            cmd_rx: self.cmd_rx,
            cancel: self.cancel,
        }
    }
}

impl<S, V> App<S, V>
where
    S: Clone + Send + Sync + 'static,
    V: View + 'static,
{
    /// Switch the embedded [`World`] into rejoin mode (see
    /// [`World::with_rejoin`]).
    ///
    /// Use for endurance / multi-hour races where mid-race reconnects should
    /// resume a prior disconnected entrant (matched by LFS.net username) rather
    /// than create a phantom duplicate. Call before [`crate::run`].
    #[must_use]
    pub fn rejoin(mut self) -> Self {
        self.world = World::with_rejoin();
        self
    }

    /// The [`World`] state mirror embedded in every app.
    ///
    /// The runtime folds each dispatch into it before any handler runs;
    /// this accessor exists to seed or inspect it before [`crate::run`].
    /// `World` is a cheap `Arc`-handle, so clone it freely.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Borrow the runtime's outbound `Sender`. Cloneable; lives for the
    /// lifetime of the `App` and remains valid through `run()`. Useful
    /// for handler constructors (like `Presence::new(sender)`) that need a
    /// `Sender` at construction time.
    pub fn sender(&self) -> &Sender {
        &self.sender
    }

    /// Borrow the runtime's [`CancellationToken`]. Cloneable; cancelling it
    /// (or any clone) requests the dispatcher exit at its next select.
    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel
    }

    /// Return a [`crate::State<S>`] containing a clone of the app's primary
    /// state.
    ///
    /// Useful before [`crate::run`] runs - the typical use is to wire up the
    /// runtime's cancel token into a field on `S` (the app's cancel token is
    /// minted inside `with_state`, so it can't be passed to `S::new` ahead
    /// of time). The clone shares any interior-mutable fields you've built
    /// into `S` (`Arc`-based), so mutations through this handle propagate.
    pub fn state(&self) -> extract::State<S> {
        extract::State(self.state.clone())
    }

    /// Register a handler at `stage`.
    ///
    /// The handler is stored in the stage's IndexMap keyed by its concrete
    /// type's `TypeId`, which means it's also available for typed
    /// extraction by other handlers (via [`crate::FromContext`] /
    /// [`crate::Svc<T>`]). Re-registering a type at the same stage
    /// overwrites the previous entry.
    ///
    /// Two flavours of handler register here uniformly:
    /// - **Stateless handlers** - plain async fns, closures, anything that
    ///   gets the blanket impl of [`Handler`].
    /// - **Stateful handlers** - structs that manually impl [`Handler`].
    ///   These are also extractable by other handlers via their type.
    #[must_use]
    pub fn handle<T, H>(mut self, stage: Stage, handler: H) -> Self
    where
        H: Handler<T, S, V> + 'static,
        T: Send + 'static,
    {
        let key = TypeId::of::<H>();
        let entry: Box<dyn ErasedHandler<S, V>> = Box::new(HandlerService::new(handler));
        match stage {
            Stage::Pre => {
                let _ = self.pre_handlers.insert(key, entry);
            },
            Stage::Update => {
                let _ = self.update_handlers.insert(key, entry);
            },
        }
        self
    }

    /// Install an [`Installable`] bundle of handlers. Equivalent to
    /// `installable.install(self)`.
    #[must_use]
    pub fn install<I: Installable<S, V>>(self, installable: I) -> Self {
        installable.install(self)
    }

    /// Spawn a background task that emits `event` as a synthetic event every
    /// `period`, until the runtime's cancel token fires.
    ///
    /// Use this for fire-and-forget periodic emitters where the event payload
    /// is static or cheaply cloneable (a unit struct, a counter, etc.).
    ///
    /// The interval uses [`tokio::time::MissedTickBehavior::Skip`]: if the
    /// runtime pauses, missed ticks are dropped rather than burst-fired.
    ///
    /// The task is spawned immediately, so `App::periodic` **must be called
    /// inside a tokio runtime context**. Events queue on the runtime's
    /// back-channel and are drained once [`crate::run`] begins.
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
