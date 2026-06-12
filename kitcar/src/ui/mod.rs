//! UI extension.
//!
//! Drives a per-connection [`Component`] tree on its own background thread
//! (necessary because `taffy` layout isn't `Send`, so view tasks run on a
//! `LocalSet`). Registers as a [`Handler`] so the runtime forwards
//! `Ncn` / `Cnl` / `Btc` / `Btt` / `Bfn` packets into the UI thread, and
//! as an extractor so handlers can pull the [`Ui`] handle by value to push
//! global state updates / per-player state / view messages.
//!
//! For PoC purposes the root [`Component`] is fixed at construction. Swapping
//! UIs at runtime is intentionally out of scope.
//!
//! ```ignore
//! let app = App::new();
//! let ui = Ui::<MyView, GlobalProps, PlayerProps>::new(
//!     app.sender().clone(),
//!     initial_global,
//!     |ucid, invalidator| MyView::new(ucid, invalidator),
//! );
//! let app = app
//!     .with_state(...)
//!     .extension(ui)
//!     .handler(my_handler);
//!
//! async fn my_handler(ui: Ui<MyView, GlobalProps, PlayerProps>) -> Result<(), AppError> {
//!     ui.assign(new_global_props);
//!     Ok(())
//! }
//! ```

use insim::identifiers::ConnectionId;
use insim_extra::ui::Ui as InnerUi;
pub use insim_extra::ui::{
    Canvas, CanvasDiff, Component, InvalidateHandle, Node, NodeKind, TypeInMapper, UiError,
    background, clickable, container, empty, text, typein,
};
use tokio::sync::{broadcast, mpsc};

use crate::{Dispatch, ExtractCx, FromContext, Handler, Sender};

/// UI handle. Construct with [`Ui::new`] and register via
/// [`crate::App::install`]. Pulled into handlers via [`FromContext`].
pub struct Ui<Cmp, G, C>
where
    Cmp: Component + 'static,
{
    inner: InnerUi<Cmp, G, C>,
}

impl<Cmp, G, C> Clone for Ui<Cmp, G, C>
where
    Cmp: Component + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Cmp, G, C> std::fmt::Debug for Ui<Cmp, G, C>
where
    Cmp: Component + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ui").finish_non_exhaustive()
    }
}

impl<Cmp, G, C> Ui<Cmp, G, C>
where
    Cmp: Component + 'static,
    for<'a> Cmp::Props<'a>: From<(&'a G, &'a C)>,
    G: Send + Sync + 'static,
    C: Clone + Send + Sync + Default + 'static,
{
    /// Create a new UI handle and spawn its background LocalSet thread.
    ///
    /// `sender` is the runtime back-channel - get it from
    /// [`App::sender`](crate::App::sender) **before** registering the `Ui` via
    /// `.extension(...)`. `initial_global` is the starting value for the
    /// global props; update later via [`Ui::assign_global`]. `make_root` is
    /// called once per connection when an `Ncn` packet arrives.
    pub fn new<F>(sender: Sender, initial_global: G, make_root: F) -> Self
    where
        F: FnMut(ConnectionId, InvalidateHandle) -> Cmp + Send + 'static,
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

    /// Replace the global props. All active views re-render against the new value.
    pub fn assign_global(&self, value: G) {
        self.inner.assign_global(value);
    }

    /// Apply a closure to the current global props in place.
    pub fn modify<F: FnOnce(&mut G)>(&self, f: F)
    where
        G: Clone,
    {
        self.inner.modify(f);
    }

    /// Push per-connection props to a specific player's view.
    pub async fn assign_player(
        &self,
        ucid: ConnectionId,
        props: C,
    ) -> Result<(), mpsc::error::SendError<(ConnectionId, C)>> {
        self.inner.assign_player(ucid, props).await
    }

    /// Push an external message into a specific player's view.
    pub async fn update(
        &self,
        ucid: ConnectionId,
        msg: Cmp::Message,
    ) -> Result<(), mpsc::error::SendError<(ConnectionId, Cmp::Message)>> {
        self.inner.update(ucid, msg).await
    }

    /// Forward a wire packet into the UI thread.
    pub fn forward_packet(&self, p: &insim::Packet) {
        self.inner.forward_packet(p);
    }
}

impl<S, Cmp, G, C> FromContext<S> for Ui<Cmp, G, C>
where
    Cmp: Component + 'static,
    for<'a> Cmp::Props<'a>: From<(&'a G, &'a C)>,
    G: Send + Sync + 'static,
    C: Clone + Send + Sync + Default + 'static,
{
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.lookup::<Ui<Cmp, G, C>>()
    }
}

impl<S, Cmp, G, C> Handler<(), S> for Ui<Cmp, G, C>
where
    S: Send + Sync + 'static,
    Cmp: Component + 'static,
    for<'a> Cmp::Props<'a>: From<(&'a G, &'a C)>,
    G: Send + Sync + 'static,
    C: Clone + Send + Sync + Default + 'static,
{
    fn call(
        self,
        cx: &ExtractCx<'_, S>,
    ) -> impl std::future::Future<Output = Result<(), crate::AppError>> + Send {
        let d = cx.dispatch.clone();
        async move {
            if let Dispatch::Packet(p) = d {
                self.forward_packet(&p);
            }
            Ok(())
        }
    }
}
