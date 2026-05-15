//! UI extension.
//!
//! Drives a per-connection [`Component`] tree on its own background thread
//! (necessary because `taffy` layout isn't `Send`, so view tasks run on a
//! `LocalSet`). Registers as an [`Extension`] so the runtime forwards
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

pub mod canvas;
pub mod id_pool;
mod node;
mod view;

use std::{collections::HashMap, sync::Arc};

pub use canvas::{Canvas, CanvasDiff};
use insim::{
    Packet, WithRequestId,
    identifiers::ConnectionId,
    insim::{Bfn, BfnType, TinyType},
};
pub use node::*;
use tokio::{
    sync::{Notify, mpsc, watch},
    task::LocalSet,
};
pub use view::{Component, InvalidateHandle};
use view::{RunViewArgs, ViewInput, run_view};

use crate::{
    event::Dispatch,
    extract::{ExtractCx, FromContext, Sender},
    middleware::{EventCx, Extension},
};

/// Errors from the UI subsystem.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UiError {
    /// Failed to create the UI thread's tokio runtime.
    #[error("Failed to create UI runtime")]
    RuntimeCreationFailed,
}

// Per-active-connection channels owned by the mount loop.
struct ActiveViewChannels<M: Clone + Send + 'static, C: Clone + Send + Sync + 'static> {
    connection_props_tx: watch::Sender<C>,
    view_event_tx: mpsc::UnboundedSender<ViewInput<M>>,
}

struct UiInner<Cmp, G, C>
where
    Cmp: Component,
{
    // Senders exposed to the user via [`Ui`] methods.
    global: watch::Sender<Arc<G>>,
    connection_props: mpsc::Sender<(ConnectionId, C)>,
    view_messages: mpsc::Sender<(ConnectionId, Cmp::Message)>,
    // Packet forward path from `Extension::on_event` into the thread.
    packet_tx: mpsc::UnboundedSender<Packet>,
}

/// UI handle. Construct with [`Ui::new`] and register via
/// [`crate::App::extension`]. Pulled into handlers via [`FromContext`].
pub struct Ui<Cmp, G, C>
where
    Cmp: Component + 'static,
{
    inner: Arc<UiInner<Cmp, G, C>>,
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
    /// global props; update later via [`Ui::assign`]. `make_root` is called
    /// once per connection when an `Ncn` packet arrives, producing the root
    /// [`Component`] for that player.
    pub fn new<F>(sender: Sender, initial_global: G, make_root: F) -> Self
    where
        F: FnMut(ConnectionId, InvalidateHandle) -> Cmp + Send + 'static,
    {
        let (global, _) = watch::channel(Arc::new(initial_global));
        let (connection_props, connection_props_rx) = mpsc::channel(100);
        let (view_messages, view_msg_rx) = mpsc::channel(100);
        let (packet_tx, packet_rx) = mpsc::unbounded_channel();

        spawn_ui_thread::<Cmp, G, C, F>(
            global.clone(),
            make_root,
            connection_props_rx,
            view_msg_rx,
            packet_rx,
            sender,
        );

        Self {
            inner: Arc::new(UiInner {
                global,
                connection_props,
                view_messages,
                packet_tx,
            }),
        }
    }

    /// Replace the global props. All active views re-render against the new value.
    pub fn assign(&self, value: G) {
        let _ = self.inner.global.send_replace(Arc::new(value));
    }

    /// Apply a closure to the current global props in place. Equivalent to
    /// reading the current value, mutating it, then [`Ui::assign`]'ing the
    /// result - but the closure-form avoids the read step and, more
    /// importantly, lets two handlers update *different fields* of `G`
    /// without clobbering each other (each `assign(full_g)` would have the
    /// last writer win on every field; `modify` lets each writer touch only
    /// its own field).
    ///
    /// Internally uses [`tokio::sync::watch::Sender::send_modify`] plus
    /// [`std::sync::Arc::make_mut`] for cheap clone-on-write under the
    /// shared `Arc<G>`.
    pub fn modify<F: FnOnce(&mut G)>(&self, f: F)
    where
        G: Clone,
    {
        self.inner.global.send_modify(|arc| f(Arc::make_mut(arc)));
    }

    /// Push per-connection props to a specific player's view.
    pub async fn set_player_state(
        &self,
        ucid: ConnectionId,
        props: C,
    ) -> Result<(), mpsc::error::SendError<(ConnectionId, C)>> {
        self.inner.connection_props.send((ucid, props)).await
    }

    /// Push an external message into a specific player's view (calls
    /// [`Component::update`] on its instance).
    pub async fn update(
        &self,
        ucid: ConnectionId,
        msg: Cmp::Message,
    ) -> Result<(), mpsc::error::SendError<(ConnectionId, Cmp::Message)>> {
        self.inner.view_messages.send((ucid, msg)).await
    }
}

impl<S, Cmp, G, C> Extension<S> for Ui<Cmp, G, C>
where
    S: Send + Sync + 'static,
    Cmp: Component + 'static,
    for<'a> Cmp::Props<'a>: From<(&'a G, &'a C)>,
    G: Send + Sync + 'static,
    C: Clone + Send + Sync + Default + 'static,
{
    async fn on_event(&self, cx: &mut EventCx<'_, S>) {
        // Forward packets the UI cares about into the LocalSet thread.
        if let Dispatch::Packet(p) = cx.dispatch {
            match p {
                Packet::Ncn(_)
                | Packet::Cnl(_)
                | Packet::Btc(_)
                | Packet::Btt(_)
                | Packet::Bfn(_) => {
                    let _ = self.inner.packet_tx.send(p.clone());
                },
                _ => {},
            }
        }
    }
}

impl<S, Cmp, G, C> FromContext<S> for Ui<Cmp, G, C>
where
    S: Send + Sync + 'static,
    Cmp: Component + 'static,
    for<'a> Cmp::Props<'a>: From<(&'a G, &'a C)>,
    G: Send + Sync + 'static,
    C: Clone + Send + Sync + Default + 'static,
{
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.extensions.get::<Ui<Cmp, G, C>>()
    }
}

fn spawn_ui_thread<Cmp, G, C, F>(
    global: watch::Sender<Arc<G>>,
    make_root: F,
    mut connection_props_rx: mpsc::Receiver<(ConnectionId, C)>,
    mut view_msg_rx: mpsc::Receiver<(ConnectionId, Cmp::Message)>,
    mut packet_rx: mpsc::UnboundedReceiver<Packet>,
    sender: Sender,
) where
    Cmp: Component + 'static,
    for<'a> Cmp::Props<'a>: From<(&'a G, &'a C)>,
    G: Send + Sync + 'static,
    C: Clone + Send + Sync + Default + 'static,
    F: FnMut(ConnectionId, InvalidateHandle) -> Cmp + Send + 'static,
{
    // Own thread because Taffy isn't Send and view tasks must run on a LocalSet.
    let _ = std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                tracing::error!("Failed to create UI runtime: {e}");
                return;
            },
        };

        let local = LocalSet::new();
        local.block_on(&rt, async move {
            let mut make_root = make_root;
            let mut active: HashMap<ConnectionId, ActiveViewChannels<Cmp::Message, C>> =
                HashMap::new();
            let mut view_msg_rx_closed = false;
            let mut connection_props_rx_closed = false;
            let mut packet_rx_closed = false;

            // Ask LFS for the current connection list so the UI catches up on
            // bot restart against a running server.
            if let Err(e) = sender.packet(TinyType::Ncn.with_request_id(1)) {
                tracing::warn!("UI: failed to request current connections: {e}");
            }

            loop {
                tokio::select! {
                    pkt = packet_rx.recv(), if !packet_rx_closed => match pkt {
                        Some(Packet::Ncn(ncn)) => {
                            if active.contains_key(&ncn.ucid) {
                                continue;
                            }
                            let invalidation_notify = Arc::new(Notify::new());
                            let root = make_root(
                                ncn.ucid,
                                InvalidateHandle::new(invalidation_notify.clone()),
                            );
                            let (props_tx, props_rx) = watch::channel(C::default());
                            let (event_tx, event_rx) = mpsc::unbounded_channel();
                            run_view(RunViewArgs {
                                ucid: ncn.ucid,
                                root,
                                invalidation_notify,
                                global_props: global.subscribe(),
                                connection_props: props_rx,
                                view_event_rx: event_rx,
                                sender: sender.clone(),
                            });
                            let _ = active.insert(
                                ncn.ucid,
                                ActiveViewChannels {
                                    connection_props_tx: props_tx,
                                    view_event_tx: event_tx,
                                },
                            );
                        }
                        Some(Packet::Cnl(cnl)) => {
                            let _ = active.remove(&cnl.ucid);
                        }
                        Some(Packet::Btc(btc)) => {
                            if let Some(c) = active.get(&btc.ucid) {
                                let _ = c.view_event_tx.send(ViewInput::Click { clickid: btc.clickid });
                            }
                        }
                        Some(Packet::Btt(btt)) => {
                            if let Some(c) = active.get(&btt.ucid) {
                                let _ = c.view_event_tx.send(ViewInput::TypeIn {
                                    clickid: btt.clickid,
                                    text: btt.text,
                                });
                            }
                        }
                        Some(Packet::Bfn(bfn)) => {
                            if let Some(c) = active.get(&bfn.ucid) {
                                let _ = c.view_event_tx.send(ViewInput::Bfn { subt: bfn.subt });
                            }
                        }
                        Some(_) => {}
                        None => {
                            packet_rx_closed = true;
                        }
                    },

                    res = connection_props_rx.recv(), if !connection_props_rx_closed => match res {
                        Some((ucid, props)) => {
                            if let Some(c) = active.get_mut(&ucid) {
                                let _ = c.connection_props_tx.send(props);
                            }
                        }
                        None => {
                            connection_props_rx_closed = true;
                        }
                    },

                    res = view_msg_rx.recv(), if !view_msg_rx_closed => match res {
                        Some((ucid, msg)) => {
                            if let Some(c) = active.get(&ucid) {
                                let _ = c.view_event_tx.send(ViewInput::Message(msg));
                            }
                        }
                        None => {
                            view_msg_rx_closed = true;
                        }
                    },
                }

                // If everything has hung up, shut down.
                if packet_rx_closed && connection_props_rx_closed && view_msg_rx_closed {
                    break;
                }
            }

            // Clear remaining buttons across all tracked connections on shutdown.
            let clears: Vec<Bfn> = active
                .drain()
                .map(|(ucid, _)| Bfn {
                    ucid,
                    subt: BfnType::Clear,
                    ..Default::default()
                })
                .collect();
            for bfn in clears {
                if let Err(e) = sender.packet(bfn) {
                    tracing::warn!("UI shutdown clear failed: {e}");
                }
            }
        });
    });
}

// ---------------------------------------------------------------------------
// Free-function builders for ergonomic Node tree construction.
// ---------------------------------------------------------------------------

/// Empty container node - useful as a layout root.
pub fn container<Msg>() -> node::Node<Msg> {
    node::Node::container()
}

/// Filled background node (no text).
pub fn background<Msg>(bstyle: insim::insim::BtnStyle) -> node::Node<Msg> {
    node::Node::background(bstyle)
}

/// Clickable button that emits `msg` when pressed.
pub fn clickable<Msg>(
    text: impl Into<String>,
    bstyle: insim::insim::BtnStyle,
    msg: Msg,
) -> node::Node<Msg> {
    node::Node::clickable(text, bstyle, msg)
}

/// Static text node.
pub fn text<Msg>(text: impl Into<String>, bstyle: insim::insim::BtnStyle) -> node::Node<Msg> {
    node::Node::text(text, bstyle)
}

/// Typeable input node - `mapper` converts the typed string to a message.
pub fn typein<Msg, F>(
    text: impl Into<String>,
    bstyle: insim::insim::BtnStyle,
    limit: u8,
    mapper: F,
) -> node::Node<Msg>
where
    F: Fn(String) -> Msg + 'static,
{
    node::Node::typein(text, bstyle, limit, mapper)
}

/// Zero-sized placeholder node.
pub fn empty<Msg>() -> node::Node<Msg> {
    node::Node::empty()
}
