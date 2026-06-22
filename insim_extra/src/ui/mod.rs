//! UI component system.
//!
//! Drives a per-connection [`Component`] tree on its own background thread
//! (necessary because `taffy` layout isn't `Send`, so view tasks run on a
//! `LocalSet`). Forwards `Ncn` / `Cnl` / `Btc` / `Btt` / `Bfn` packets into
//! the UI thread via [`Ui::forward_packet`].
//!
//! # Multiplayer only
//!
//! [`Ui`] is designed exclusively for **dedicated multiplayer servers**. The
//! host/local connection ([`ConnectionId::LOCAL`] / UCID 0) is silently ignored:
//! no [`Component`] is mounted for it and no buttons are ever sent to it.
//!
//! If you are building a **local / single-player** tool (i.e. connecting with
//! `IsiFlags::LOCAL` or targeting UCID 0 directly), use [`Canvas`] and
//! [`Component`] yourself rather than going through [`Ui`].
//!
//! ```ignore
//! use tokio::sync::mpsc;
//! let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel();
//! let ui = Ui::<MyView>::new(outgoing_tx, initial_global);
//! // Each connection's view is built via `MyView::mount` when its `Ncn` arrives.
//!
//! // Bridge outgoing button packets to your connection
//! tokio::spawn(async move {
//!     while let Some(p) = outgoing_rx.recv().await { connection.send(p).await; }
//! });
//!
//! // Subscribe to click/type-in events
//! let mut rx = ui.subscribe();
//! tokio::spawn(async move {
//!     while let Ok(msg) = rx.recv().await { println!("UI event: {msg:?}"); }
//! });
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
    sync::{Notify, broadcast, mpsc, watch},
    task::LocalSet,
};
pub use view::{Component, InvalidateHandle, View};
use view::{RunViewArgs, ViewInput, run_view};

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

struct UiInner<V>
where
    V: View,
{
    global: watch::Sender<Arc<V::Global>>,
    connection_props: mpsc::Sender<(ConnectionId, V::Connection)>,
    view_messages: mpsc::Sender<(ConnectionId, V::Message)>,
    // Inbound: wire packets from LFS into the UI thread.
    packet_tx: mpsc::UnboundedSender<Packet>,
    // Outbound: click/type-in events broadcast to subscribers.
    message_tx: broadcast::Sender<V::Message>,
}

/// UI handle for dedicated multiplayer servers.
///
/// Construct with [`Ui::new`] and forward wire packets via
/// [`Ui::forward_packet`]. Subscribe to click/type-in events via
/// [`Ui::subscribe`].
///
/// [`ConnectionId::LOCAL`] (UCID 0) is always ignored - no [`Component`] is
/// mounted for it. For local / single-player tools, use [`Canvas`] directly.
pub struct Ui<V>
where
    V: View + 'static,
{
    inner: Arc<UiInner<V>>,
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
    /// `outgoing_tx` receives button render packets that should be forwarded to
    /// the InSim connection. `initial_global` is the starting global state value.
    /// Each connection's view is constructed via [`View::mount`] when its `Ncn`
    /// packet is seen.
    pub fn new(outgoing_tx: mpsc::UnboundedSender<Packet>, initial_global: V::Global) -> Self {
        let (global, _) = watch::channel(Arc::new(initial_global));
        let (connection_props, connection_props_rx) = mpsc::channel(100);
        let (view_messages, view_msg_rx) = mpsc::channel(100);
        let (packet_tx, packet_rx) = mpsc::unbounded_channel();
        let (message_tx, _) = broadcast::channel(64);

        spawn_ui_thread::<V>(
            global.clone(),
            connection_props_rx,
            view_msg_rx,
            packet_rx,
            outgoing_tx,
            message_tx.clone(),
        );

        Self {
            inner: Arc::new(UiInner {
                global,
                connection_props,
                view_messages,
                packet_tx,
                message_tx,
            }),
        }
    }

    /// Subscribe to click and type-in events produced by any active view.
    pub fn subscribe(&self) -> broadcast::Receiver<V::Message> {
        self.inner.message_tx.subscribe()
    }

    /// Replace the global state. All active views re-render against the new value.
    pub fn assign_global(&self, value: V::Global) {
        let _ = self.inner.global.send_replace(Arc::new(value));
    }

    /// Apply a closure to the current global state in place.
    pub fn modify<F: FnOnce(&mut V::Global)>(&self, f: F)
    where
        V::Global: Clone,
    {
        self.inner.global.send_modify(|arc| f(Arc::make_mut(arc)));
    }

    /// Push per-connection state to a specific player's view.
    pub async fn assign_player(
        &self,
        ucid: ConnectionId,
        props: V::Connection,
    ) -> Result<(), mpsc::error::SendError<(ConnectionId, V::Connection)>> {
        self.inner.connection_props.send((ucid, props)).await
    }

    /// Push an external message into a specific player's view.
    pub async fn update(
        &self,
        ucid: ConnectionId,
        msg: V::Message,
    ) -> Result<(), mpsc::error::SendError<(ConnectionId, V::Message)>> {
        self.inner.view_messages.send((ucid, msg)).await
    }

    /// Forward a wire packet into the UI thread if it's one the UI cares
    /// about (Ncn/Cnl/Btc/Btt/Bfn). Other packets are ignored.
    pub fn forward_packet(&self, p: &Packet) {
        match p {
            Packet::Ncn(_) | Packet::Cnl(_) | Packet::Btc(_) | Packet::Btt(_) | Packet::Bfn(_) => {
                let _ = self.inner.packet_tx.send(p.clone());
            },
            _ => {},
        }
    }
}

fn spawn_ui_thread<V>(
    global: watch::Sender<Arc<V::Global>>,
    mut connection_props_rx: mpsc::Receiver<(ConnectionId, V::Connection)>,
    mut view_msg_rx: mpsc::Receiver<(ConnectionId, V::Message)>,
    mut packet_rx: mpsc::UnboundedReceiver<Packet>,
    outgoing_tx: mpsc::UnboundedSender<Packet>,
    message_tx: broadcast::Sender<V::Message>,
) where
    V: View + 'static,
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
            let mut active: HashMap<ConnectionId, ActiveViewChannels<V::Message, V::Connection>> =
                HashMap::new();
            let mut view_msg_rx_closed = false;
            let mut connection_props_rx_closed = false;
            let mut packet_rx_closed = false;

            // Ask LFS for the current connection list so the UI catches up on
            // bot restart against a running server.
            if outgoing_tx.send(TinyType::Ncn.with_request_id(1).into()).is_err() {
                tracing::warn!("UI: failed to request current connections");
            }

            loop {
                tokio::select! {
                    pkt = packet_rx.recv(), if !packet_rx_closed => match pkt {
                        Some(Packet::Ncn(ncn)) => {
                            if ncn.ucid.local() || active.contains_key(&ncn.ucid) {
                                continue;
                            }
                            let invalidation_notify = Arc::new(Notify::new());
                            let root = V::mount(
                                ncn.ucid,
                                InvalidateHandle::new(invalidation_notify.clone()),
                            );
                            let (props_tx, props_rx) = watch::channel(V::Connection::default());
                            let (event_tx, event_rx) = mpsc::unbounded_channel();
                            run_view(RunViewArgs {
                                ucid: ncn.ucid,
                                root,
                                invalidation_notify,
                                global_props: global.subscribe(),
                                connection_props: props_rx,
                                view_event_rx: event_rx,
                                outgoing_tx: outgoing_tx.clone(),
                                message_tx: message_tx.clone(),
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
                if outgoing_tx.send(Packet::from(bfn)).is_err() {
                    tracing::warn!("UI shutdown clear failed");
                }
            }
        });
    });
}

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
