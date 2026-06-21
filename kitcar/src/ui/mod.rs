//! UI extension.
//!
//! Drives a per-connection [`View`] tree on its own background thread (necessary
//! because `taffy` layout isn't `Send`, so view tasks run on a `LocalSet`). The
//! UI is a **first-class part of the app**: register it once with
//! [`App::with_ui`](crate::App::with_ui) and the runtime forwards
//! `Ncn` / `Cnl` / `Btc` / `Btt` / `Bfn` packets into the UI thread each cycle,
//! before handlers run. Handlers pull the [`Ui`] handle by value (via
//! [`FromContext`]) to push global state / per-player state / view messages.
//!
//! For PoC purposes the root [`View`] is fixed at construction. Swapping UIs at
//! runtime is intentionally out of scope.
//!
//! ```ignore
//! let app = App::new();
//! let ui = Ui::<MyView>::new(
//!     app.sender().clone(),
//!     initial_global,
//!     |ucid, invalidator| MyView::new(ucid, invalidator),
//! );
//! let app = app
//!     .with_ui(ui)
//!     .handle(Stage::Update, my_handler);
//!
//! async fn my_handler(ui: Ui<MyView>) -> Result<(), AppError> {
//!     ui.assign_global(new_global_state);
//!     Ok(())
//! }
//! ```

use std::any::Any;

use insim::identifiers::ConnectionId;
use insim_extra::ui::Ui as InnerUi;
pub use insim_extra::ui::{
    Canvas, CanvasDiff, Component, InvalidateHandle, Node, NodeKind, TypeInMapper, UiError, View,
    background, clickable, container, empty, text, typein,
};
use tokio::sync::{broadcast, mpsc};

use crate::{ExtractCx, FromContext, Sender};

/// UI handle. Construct with [`Ui::new`] and register via
/// [`crate::App::with_ui`]. Pulled into handlers via [`FromContext`].
pub struct Ui<V>
where
    V: View + 'static,
{
    inner: InnerUi<V>,
}

impl<V> Clone for Ui<V>
where
    V: View + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<V> std::fmt::Debug for Ui<V>
where
    V: View + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ui").finish_non_exhaustive()
    }
}

impl<V> Ui<V>
where
    V: View + 'static,
{
    /// Create a new UI handle and spawn its background LocalSet thread.
    ///
    /// `sender` is the runtime back-channel - get it from
    /// [`App::sender`](crate::App::sender) **before** registering the `Ui` via
    /// [`with_ui`](crate::App::with_ui). `initial_global` is the starting value
    /// for the global state; update later via [`Ui::assign_global`]. `make_root`
    /// is called once per connection when an `Ncn` packet arrives.
    pub fn new<F>(sender: Sender, initial_global: V::Global, make_root: F) -> Self
    where
        F: FnMut(ConnectionId, InvalidateHandle) -> V + Send + 'static,
    {
        let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel::<insim::Packet>();
        let inner = InnerUi::new(outgoing_tx, initial_global, make_root);

        // Forward outgoing UI button packets to LFS via sender.
        {
            let sender = sender.clone();
            drop(tokio::spawn(async move {
                while let Some(p) = outgoing_rx.recv().await {
                    if let Err(e) = sender.packet(p) {
                        tracing::warn!("UI packet forward failed: {e}");
                        break;
                    }
                }
            }));
        }

        // Broadcast click/type-in events into the app event bus.
        {
            let mut rx = inner.subscribe();
            drop(tokio::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(msg) => {
                            let _ = sender.event(msg);
                        },
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            }));
        }

        Self { inner }
    }

    /// Replace the global state. All active views re-render against the new value.
    pub fn assign_global(&self, value: V::Global) {
        self.inner.assign_global(value);
    }

    /// Apply a closure to the current global state in place.
    pub fn modify<F: FnOnce(&mut V::Global)>(&self, f: F)
    where
        V::Global: Clone,
    {
        self.inner.modify(f);
    }

    /// Push per-connection state to a specific player's view.
    pub async fn assign_player(
        &self,
        ucid: ConnectionId,
        props: V::Connection,
    ) -> Result<(), mpsc::error::SendError<(ConnectionId, V::Connection)>> {
        self.inner.assign_player(ucid, props).await
    }

    /// Push an external message into a specific player's view.
    pub async fn update(
        &self,
        ucid: ConnectionId,
        msg: V::Message,
    ) -> Result<(), mpsc::error::SendError<(ConnectionId, V::Message)>> {
        self.inner.update(ucid, msg).await
    }

    /// Forward a wire packet into the UI thread.
    pub fn forward_packet(&self, p: &insim::Packet) {
        self.inner.forward_packet(p);
    }
}

/// Object-safe driver the runtime holds in [`App`](crate::App)'s UI slot so it can
/// forward packets to the one mounted UI without naming its view type, and hand
/// the concrete [`Ui<V>`] back to handlers via downcast.
pub trait UiSink: Send + Sync {
    /// Forward a wire packet into the UI thread.
    fn forward_packet(&self, p: &insim::Packet);
    /// Recover the concrete `Ui<V>` for the [`FromContext`] extractor.
    fn as_any(&self) -> &dyn Any;
}

impl<V: View + 'static> UiSink for Ui<V> {
    fn forward_packet(&self, p: &insim::Packet) {
        self.inner.forward_packet(p);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<S, V> FromContext<S> for Ui<V>
where
    V: View + 'static,
{
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.ui?.as_any().downcast_ref::<Ui<V>>().cloned()
    }
}
