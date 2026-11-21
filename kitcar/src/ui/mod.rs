//! UI extension.
//!
//! Drives a per-connection [`View`] tree on its own background thread (necessary
//! because `taffy` layout isn't `Send`, so view tasks run on a `LocalSet`). The
//! UI is a **first-class part of the app**, parameterising [`App<S, V>`](crate::App)
//! alongside the state `S` - exactly like the intrinsic [`World`](crate::World).
//! Register it with [`App::with_ui`](crate::App::with_ui) and the runtime
//! attaches it to the connection when the app runs: it follows `Ncn` / `Cnl` /
//! `Btc` / `Btt` / `Bfn` packets through a tap, on its own thread. Handlers
//! pull the [`Ui<V>`] handle by value (via [`FromContext`]); because the view
//! type is fixed in the app's type, that extraction is infallible and asking for
//! the wrong view type is a compile error.
//!
//! An app that never calls `with_ui` keeps the default [`NoView`]: its `Ui` is
//! inert (no thread, all methods no-op).
//!
//! ```ignore
//! let app = App::with_state(state)
//!     .with_ui::<MyView>(initial_global)
//!     .handle(my_handler);
//!
//! async fn my_handler(ui: Ui<MyView>) -> Result<(), AppError> {
//!     ui.assign_global(new_global_state);
//!     Ok(())
//! }
//! ```

use insim::identifiers::ConnectionId;
use insim_extra::ui::Ui as InnerUi;
pub use insim_extra::ui::{
    Canvas, CanvasDiff, Component, InvalidateHandle, Node, NodeKind, TypeInMapper, UiError, View,
    ViewHandle, background, clickable, col, container, empty, row, text, typein,
};
use tokio::sync::{broadcast, mpsc};

use crate::{ExtractCx, FromContext, Sender};

/// The default [`View`] for an app with no UI. Mounts nothing and renders empty;
/// its [`Ui`] is constructed inert (see [`Ui::disabled`]), so a UI-less app pays
/// no UI thread.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoView;

impl Component for NoView {
    type Message = ();
    type Props<'a> = ();

    fn render(&self, _props: Self::Props<'_>) -> Node<Self::Message> {
        Node::empty()
    }
}

impl View for NoView {
    type Global = ();
    type Connection = ();

    fn mount(_ucid: ConnectionId, _handle: ViewHandle<Self::Message>) -> Self {
        NoView
    }

    fn props<'a>(_global: &'a (), _connection: &'a ()) -> Self::Props<'a> {}
}

/// UI handle, parameterised by its root [`View`]. Built by
/// [`App::with_ui`](crate::App::with_ui) and pulled into handlers via
/// [`FromContext`]. The [`NoView`] handle is inert.
pub struct Ui<V>
where
    V: View + 'static,
{
    inner: Option<InnerUi<V>>,
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
        f.debug_struct("Ui")
            .field("active", &self.inner.is_some())
            .finish()
    }
}

impl<V> Ui<V>
where
    V: View + 'static,
{
    /// An inert UI handle: no background thread, every method a no-op. Used as
    /// the [`NoView`] default for apps that never call
    /// [`App::with_ui`](crate::App::with_ui).
    pub fn disabled() -> Self {
        Self { inner: None }
    }

    /// Attach a live UI to the connection and spawn its background LocalSet
    /// thread. Called by the runtime; clicks and type-ins become events on
    /// `events`.
    pub(crate) async fn attach(
        connection: &insim::net::Sender,
        events: Sender,
        initial_global: V::Global,
    ) -> Result<Self, crate::AppError> {
        let inner = InnerUi::attach(connection, initial_global).await?;

        // Broadcast click/type-in events into the app event bus.
        let mut rx = inner.subscribe();
        drop(tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        let _ = events.event(msg);
                    },
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }));

        Ok(Self { inner: Some(inner) })
    }

    /// Replace the global state. All active views re-render against the new value.
    pub fn assign_global(&self, value: V::Global) {
        if let Some(inner) = &self.inner {
            inner.assign_global(value);
        }
    }

    /// Apply a closure to the current global state in place.
    pub fn modify<F: FnOnce(&mut V::Global)>(&self, f: F)
    where
        V::Global: Clone,
    {
        if let Some(inner) = &self.inner {
            inner.modify(f);
        }
    }

    /// Push per-connection state to a specific player's view.
    pub async fn assign_player(
        &self,
        ucid: ConnectionId,
        props: V::Connection,
    ) -> Result<(), mpsc::error::SendError<(ConnectionId, V::Connection)>> {
        match &self.inner {
            Some(inner) => inner.assign_player(ucid, props).await,
            None => Ok(()),
        }
    }

    /// Push an external message into a specific player's view.
    pub async fn update(
        &self,
        ucid: ConnectionId,
        msg: V::Message,
    ) -> Result<(), mpsc::error::SendError<(ConnectionId, V::Message)>> {
        match &self.inner {
            Some(inner) => inner.update(ucid, msg).await,
            None => Ok(()),
        }
    }
}

/// Infallible, type-checked extraction: the view type is fixed in the app's
/// `App<S, V>`, so `ui: Ui<V>` always resolves and asking for the wrong view
/// type fails to compile (no matching impl).
impl<S, V> FromContext<S, V> for Ui<V>
where
    V: View + 'static,
{
    fn from_context(cx: &ExtractCx<'_, S, V>) -> Option<Self> {
        Some(cx.ui.clone())
    }
}
