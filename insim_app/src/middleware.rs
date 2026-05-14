//! Extension trait, runtime context, and the bundled `Presence` /
//! `ChatParser<C>` helpers.
//!
//! An [`Extension<S>`] is a registered value that (a) handlers can pull out
//! via [`FromContext`] and (b) optionally observes every [`Dispatch`] through
//! the default-overridable `on_event` hook. Pure data extensions accept the
//! default no-op; ones that need to react (parse chat, track presence) override.
//! Either way, registration is a single `App::extension(value)` call.

use std::{
    any::Any,
    collections::HashMap,
    future::Future,
    marker::PhantomData,
    str::FromStr,
    sync::{Arc, RwLock},
};

use futures::future::BoxFuture;
use insim::{
    identifiers::ConnectionId,
    insim::{MsoUserType, Ncn},
};
use tokio_util::sync::CancellationToken;

use crate::{
    event::Dispatch,
    extensions::Extensions,
    extract::{ExtractCx, FromContext, Sender},
};

/// Context handed to an extension's `on_event`.
#[derive(Debug)]
pub struct EventCx<'a, S> {
    /// The current dispatch.
    pub dispatch: &'a Dispatch,
    /// Shared app state (read-only in the PoC).
    pub state: &'a S,
    /// Back-channel - same surface handlers see. Use `cx.sender.packet(...)` to
    /// send a wire packet, `cx.sender.event(...)` to inject a synthetic event
    /// into a subsequent dispatch cycle.
    pub sender: &'a Sender,
    /// Extension registry; same instance handlers see.
    pub extensions: &'a Extensions,
    /// Cooperative-shutdown token.
    pub cancel: &'a CancellationToken,
}

impl<'a, S> EventCx<'a, S> {
    /// Request graceful shutdown of the runtime.
    pub fn shutdown(&self) {
        self.cancel.cancel();
    }

    /// Whether shutdown has been requested.
    pub fn is_shutdown(&self) -> bool {
        self.cancel.is_cancelled()
    }
}

/// A value registered with the [`App`](crate::App). Combines two roles:
///
/// 1. **Extractor source** - every registered extension lives in the
///    [`Extensions`] registry by its `TypeId`, so handlers can pull it out via
///    [`FromContext`] (the extension author implements [`FromContext`] on the
///    type itself).
/// 2. **Optional event observer** - the default no-op `on_event` is overridden
///    by extensions that need to react to wire packets or synthetic events.
///
/// The trait takes `&self` (not `&mut self`) so the runtime can hold a single
/// `Arc<E>` shared between the registry and the dispatch chain - no `Clone`
/// bound on `E`, no per-registration clone. Stateful extensions manage their
/// own mutability internally (`Arc<RwLock<…>>` is the usual pattern).
///
/// Like [`crate::Handler`], the trait method is declared with `-> impl Future
/// + Send` so the runtime can require `Send` at the [`ErasedExtension`]
/// boundary; impls can still use `async fn` syntax.
pub trait Extension<S>: Send + Sync + 'static {
    /// Called for every dispatch (wire packets and synthetic events). Runs
    /// sequentially, in registration order, *before* handlers. Default is a
    /// no-op so pure data extensions don't have to write anything.
    fn on_event<'a>(&'a self, _cx: &'a mut EventCx<'_, S>) -> impl Future<Output = ()> + Send + 'a {
        async {}
    }
}

/// Object-safe shim so heterogeneous extensions can sit behind one Arc.
pub(crate) trait ErasedExtension<S>: Send + Sync {
    fn on_event<'a>(&'a self, cx: &'a mut EventCx<'_, S>) -> BoxFuture<'a, ()>;
}

impl<S, E> ErasedExtension<S> for E
where
    E: Extension<S>,
{
    fn on_event<'a>(&'a self, cx: &'a mut EventCx<'_, S>) -> BoxFuture<'a, ()> {
        Box::pin(<Self as Extension<S>>::on_event(self, cx))
    }
}

// ---------------------------------------------------------------------------
// Presence - tracks connections, emits Connected/Disconnected, and acts as
// its own extractor.
// ---------------------------------------------------------------------------

/// Per-connection record stored by [`Presence`].
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Connection identifier.
    pub ucid: ConnectionId,
    /// LFS.net username.
    pub uname: String,
    /// Player nickname.
    pub pname: String,
    /// Whether the connection has admin privileges.
    pub admin: bool,
}

/// Synthetic event emitted by [`Presence`] when a connection joins.
#[derive(Debug, Clone)]
pub struct Connected(pub ConnectionInfo);

/// Synthetic event emitted by [`Presence`] when a connection leaves.
#[derive(Debug, Clone)]
pub struct Disconnected {
    /// The connection that left.
    pub ucid: ConnectionId,
    /// Last known info for the connection (cloned out of the live map).
    pub info: Option<ConnectionInfo>,
}

#[derive(Default)]
struct PresenceInner {
    connections: HashMap<ConnectionId, ConnectionInfo>,
}

/// Extension that tracks active connections (joins on `Ncn`, removes on
/// `Cnl`), emits [`Connected`] / [`Disconnected`] synthetic events, and is
/// queryable by handlers via [`FromContext`].
///
/// ```ignore
/// let app = App::new()
///     .with_state(MyState { ... })
///     .extension(Presence::new());
///
/// async fn handler(presence: Presence) -> Result<(), AppError> {
///     tracing::info!(count = presence.count(), "current players");
///     Ok(())
/// }
/// ```
///
/// State is shared via `Arc<RwLock<_>>`; cloning the handle is cheap and every
/// clone observes the same map.
#[derive(Clone, Default)]
pub struct Presence {
    inner: Arc<RwLock<PresenceInner>>,
}

impl std::fmt::Debug for Presence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Presence")
            .field("connections", &self.count())
            .finish()
    }
}

impl Presence {
    /// Create a new presence extension with an empty connection map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of tracked connections.
    pub fn count(&self) -> usize {
        self.inner.read().expect("poison").connections.len()
    }

    /// Look up one connection by UCID.
    pub fn get(&self, ucid: ConnectionId) -> Option<ConnectionInfo> {
        self.inner
            .read()
            .expect("poison")
            .connections
            .get(&ucid)
            .cloned()
    }

    /// Snapshot of all tracked connections.
    pub fn connections(&self) -> Vec<ConnectionInfo> {
        self.inner
            .read()
            .expect("poison")
            .connections
            .values()
            .cloned()
            .collect()
    }
}

impl<S: Send + Sync + 'static> Extension<S> for Presence {
    async fn on_event(&self, cx: &mut EventCx<'_, S>) {
        match cx.dispatch {
            Dispatch::Packet(insim::Packet::Ncn(ncn)) => {
                let Ncn {
                    ucid,
                    uname,
                    pname,
                    admin,
                    ..
                } = ncn.clone();
                let info = ConnectionInfo {
                    ucid,
                    uname,
                    pname,
                    admin,
                };
                let _ = self
                    .inner
                    .write()
                    .expect("poison")
                    .connections
                    .insert(info.ucid, info.clone());
                let _ = cx.sender.event(Connected(info));
            },
            Dispatch::Packet(insim::Packet::Cnl(cnl)) => {
                let info = self
                    .inner
                    .write()
                    .expect("poison")
                    .connections
                    .remove(&cnl.ucid);
                let _ = cx.sender.event(Disconnected {
                    ucid: cnl.ucid,
                    info,
                });
            },
            _ => {},
        }
    }
}

/// [`Presence`] is its own extractor: register via [`crate::App::extension`]
/// and any handler can take it by value.
impl<S: Send + Sync + 'static> FromContext<S> for Presence {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.extensions.get::<Presence>()
    }
}

// ---------------------------------------------------------------------------
// ChatParser<C> - typed Mso → C parser. Parses once, dispatches via Event<C>.
// ---------------------------------------------------------------------------

/// Extension that parses every `Mso` body into a typed value `C` via
/// [`FromStr`] and emits the parsed value as a synthetic event.
///
/// **The point of using this over a per-handler parse extractor is that the
/// parse runs once per `Mso` packet** regardless of how many `Event<C>`
/// handlers are registered - they all see the same `Arc`-wrapped value.
///
/// Pair with `Event<C>` handlers and the existing
/// `insim_extras::chat::Parse` derive (via a small `FromStr` bridge - see
/// docs on the example crate). On parse failure (wrong prefix, unknown
/// command, malformed args) no event is emitted.
pub struct ChatParser<C> {
    _phantom: PhantomData<fn() -> C>,
}

impl<C> ChatParser<C> {
    /// Create a new typed chat parser extension.
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<C> Default for ChatParser<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> std::fmt::Debug for ChatParser<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatParser").finish_non_exhaustive()
    }
}

impl<S, C> Extension<S> for ChatParser<C>
where
    C: FromStr + Any + Send + Sync + 'static,
    S: Send + Sync + 'static,
{
    async fn on_event(&self, cx: &mut EventCx<'_, S>) {
        let Dispatch::Packet(insim::Packet::Mso(mso)) = cx.dispatch else {
            return;
        };
        if !matches!(mso.usertype, MsoUserType::User | MsoUserType::Prefix) {
            return;
        }
        if let Ok(c) = mso.msg_from_textstart().trim().parse::<C>() {
            let _ = cx.sender.event(c);
        }
    }
}
