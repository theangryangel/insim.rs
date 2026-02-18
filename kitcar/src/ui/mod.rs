use std::collections::HashMap;

use insim::{
    Packet, WithRequestId,
    identifiers::ConnectionId,
    insim::{Bfn, BfnType, TinyType},
};
use tokio::{
    sync::{broadcast, mpsc, watch},
    task::{JoinHandle, LocalSet},
};

pub mod canvas;
pub mod id_pool;
mod node;
mod view;

pub use canvas::{Canvas, CanvasDiff};
pub use node::*;
pub use view::*;

#[derive(Debug, thiserror::Error)]
/// UiError
pub enum UiError {
    /// Failed to create UI runtime
    #[error("Failed to create UI runtime")]
    RuntimeCreationFailed,
}

// per-connection channels owned by the attach loop and forwarded into a spawned view task.
struct ActiveViewChannels<V: View> {
    // per-connection props updates (`ui::set_player_state`).
    connection_props_tx: watch::Sender<V::ConnectionState>,
    // per-connection event stream (external messages + demuxed UI input).
    view_event_tx: mpsc::UnboundedSender<view::ViewInput<V::Message>>,
}

/// Ui handle. Create using [attach] or [attach_with].
/// When dropped all insim buttons will be automatically removed.
/// Intended for multi-player/multi-connection UIs
#[derive(Debug)]
pub struct Ui<M: Clone + Send + 'static, G, C> {
    global: watch::Sender<G>,
    connection_props: mpsc::Sender<(ConnectionId, C)>,
    view_messages: mpsc::Sender<(ConnectionId, M)>,
    outbound: broadcast::Sender<(ConnectionId, M)>,
}

impl<M, G, C> Ui<M, G, C>
where
    M: Clone + Send + 'static,
    G: Clone + Send + Sync + Default + 'static,
    C: Clone + Send + Sync + Default + 'static,
{
    /// Update the global state for all connections, triggering a re-render.
    /// Global state is shared state visible to all connected players.
    pub fn set_global_state(&self, value: G) {
        let _ = self.global.send(value);
    }

    /// Update the state for a specific connection, triggering a re-render for that player.
    /// Player state is per-player state, useful for player-specific UI elements.
    pub async fn set_player_state(&self, ucid: ConnectionId, value: C) {
        let _ = self.connection_props.send((ucid, value)).await;
    }

    /// Subscribe to messages produced by user UI interactions (e.g. button click / type-in).
    pub fn subscribe(&self) -> broadcast::Receiver<(ConnectionId, M)> {
        self.outbound.subscribe()
    }

    /// Inject message into UI for a given ucid.
    pub async fn update(&self, ucid: ConnectionId, msg: M) {
        let _ = self.view_messages.send((ucid, msg)).await;
    }
}

/// Attach a UI view to an insim connection, spawning a view instance for each connected player.
/// Dropping the returned [`Ui`] handle will result in the UI being cleared for all players.
///
/// All UI tasks run on a LocalSet, so view implementations don't need to be Send to accommodate
/// taffy.
///
/// # Example
///
/// ```rust,ignore
/// struct MyView;
///
/// impl Component for MyView {
///     type Props = Props;
///     type Message = MyMsg;
///
///     fn render(&self, props: Props) -> Node<Self::Message> {
///         container()
///             .with_child(text(format!("Score: {}", props.global.score), BtnStyle::default()))
///     }
/// }
/// impl View for MyView {
///     type GlobalState = GameState;
///     type ConnectionState = PlayerState;
///
///     fn mount(_invalidator: InvalidateHandle) -> Self {
///         Self
///     }
///
///     fn compose(global: Self::GlobalState, player: Self::ConnectionState) -> Self::Props {
///         Props { global, player }
///     }
/// }
///
/// #[derive(Clone, Default)]
/// struct GameState { score: u32 }
/// #[derive(Clone, Default)]
/// struct PlayerState { ready: bool }
/// #[derive(Clone)]
/// struct Props { global: GameState, player: PlayerState }
///
/// let (ui, handle) = attach::<MyView>(insim, GameState::default());
///
/// // Update global state (re-renders for all players)
/// ui.set_global_state(GameState { score: 100 });
///
/// // Update per-player state
/// ui.set_player_state(player_ucid, PlayerState { ready: true }).await;
/// ```
#[allow(clippy::type_complexity)]
pub fn attach<V>(
    insim: insim::builder::InsimTask,
    props: V::GlobalState,
) -> (
    Ui<V::Message, V::GlobalState, V::ConnectionState>,
    JoinHandle<Result<(), UiError>>,
)
where
    V: View,
{
    let (ingress_tx, ingress_rx) = broadcast::channel::<()>(1);
    drop(ingress_tx);

    attach_with::<V, _, _>(insim, props, ingress_rx, |_| None)
}

/// Attach a UI view to an insim connection and map an external broadcast stream into
/// targeted per-player UI messages.
///
/// This is useful for feeding parsed chat commands (or any other command/event bus)
/// directly into UI state updates without an extra forwarding task.
#[allow(clippy::type_complexity)]
pub fn attach_with<V, E, F>(
    insim: insim::builder::InsimTask,
    props: V::GlobalState,
    mut ingress_rx: broadcast::Receiver<E>,
    mut map_ingress: F,
) -> (
    Ui<V::Message, V::GlobalState, V::ConnectionState>,
    JoinHandle<Result<(), UiError>>,
)
where
    V: View,
    E: Clone + Send + 'static,
    F: FnMut(E) -> Option<(ConnectionId, V::Message)> + Send + 'static,
{
    let (global_tx, _global_rx) = watch::channel(props);
    // Outside-in connection props updates (`Ui::set_player_state`).
    let (connection_props_tx, mut connection_props_rx) = mpsc::channel(100);
    // Outside-in per-player messages (`Ui::update`).
    let (view_msg_tx, mut view_msg_rx) = mpsc::channel::<(ConnectionId, V::Message)>(100);
    let (outbound_tx, _outbound_rx) = broadcast::channel::<(ConnectionId, V::Message)>(100);
    let ui_handle = Ui {
        global: global_tx.clone(),
        connection_props: connection_props_tx,
        view_messages: view_msg_tx,
        outbound: outbound_tx.clone(),
    };

    drop(_global_rx);

    // XXX: We run on our own thread because we need to use LocalSet until Taffy Style is Send.
    // https://github.com/DioxusLabs/taffy/issues/823
    let thread_handle = std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                tracing::error!("Failed to create UI runtime: {}", e);
                return Err::<(), UiError>(UiError::RuntimeCreationFailed);
            },
        };

        let local = LocalSet::new();
        local.block_on(&rt, async move {
            let mut packets = insim.subscribe();
            let mut view_msg_rx_closed = false;
            let mut ingress_rx_closed = false;
            let mut active: HashMap<ConnectionId, ActiveViewChannels<V>> = HashMap::new();

            if let Err(e) = insim.send(TinyType::Ncn.with_request_id(1)).await {
                tracing::warn!("UI attach: failed to request current connections: {e}");
            }

            loop {
                tokio::select! {
                    packet = packets.recv() => match packet {
                        Ok(Packet::Ncn(ncn)) => {
                            if active.contains_key(&ncn.ucid) {
                                continue;
                            }

                            spawn_for::<V>(
                                ncn.ucid,
                                global_tx.subscribe(),
                                &insim,
                                &mut active,
                                outbound_tx.clone(),
                            );
                        },
                        Ok(Packet::Cnl(cnl)) => {
                            // player left, remove their props sender
                            let _ = active.remove(&cnl.ucid);
                        },
                        Ok(Packet::Btc(btc)) => {
                            if let Some(channels) = active.get(&btc.ucid) {
                                let _ = channels.view_event_tx.send(view::ViewInput::Click {
                                    clickid: btc.clickid,
                                });
                            }
                        }
                        Ok(Packet::Btt(btt)) => {
                            if let Some(channels) = active.get(&btt.ucid) {
                                let _ = channels.view_event_tx.send(view::ViewInput::TypeIn {
                                    clickid: btt.clickid,
                                    text: btt.text,
                                });
                            }
                        }
                        Ok(Packet::Bfn(bfn)) => {
                            if let Some(channels) = active.get(&bfn.ucid) {
                                let _ = channels.view_event_tx.send(view::ViewInput::Bfn {
                                    subt: bfn.subt,
                                });
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!("UI attach: packet stream lagged by {skipped} packets");
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            tracing::error!("UI attach: packet stream closed");
                            break;
                        },
                        _ => {
                            // ignore unrelated packets
                        }
                    },

                    res = connection_props_rx.recv() => match res {
                        Some((ucid, props)) => {
                            if let Some(channels) = active.get_mut(&ucid) {
                                let _ = channels.connection_props_tx.send(props);
                            }
                        },
                        None => {
                            tracing::debug!("UI attach: connection props channel closed");
                            break;
                        }
                    },

                    // External message ingress is separate from lifecycle.
                    // Senders cloned via `Ui` can keep it open, but we shut down based on
                    // connection_props_rx (driven by the main Ui handle).
                    res = view_msg_rx.recv(), if !view_msg_rx_closed => match res {
                        Some((ucid, msg)) => {
                            if let Some(channels) = active.get(&ucid) {
                                let _ = channels.view_event_tx.send(view::ViewInput::Message(msg));
                            }
                        },
                        None => {
                            // All message senders dropped, but we don't
                            // shut down here—lifecycle is tied to connection_props_rx.
                            view_msg_rx_closed = true;
                        }
                    },

                    // Mapped external ingress (`attach_with`) routed into per-player views.
                    res = ingress_rx.recv(), if !ingress_rx_closed => match res {
                        Ok(event) => {
                            if let Some((ucid, msg)) = map_ingress(event)
                                && let Some(channels) = active.get(&ucid)
                            {
                                let _ = channels.view_event_tx.send(view::ViewInput::Message(msg));
                            }
                        },
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!("UI attach_with: ingress stream lagged by {skipped} events");
                        },
                        Err(broadcast::error::RecvError::Closed) => {
                            ingress_rx_closed = true;
                        },
                    },
                }
            }

            // for all player connections automatically clear all buttons
            // when we lose the UiHandle.
            // this should happen when we lose the connection_props_rx receiver.
            let clear: Vec<Bfn> = active
                .drain()
                .map(|(ucid, _)| Bfn {
                    ucid,
                    subt: BfnType::Clear,
                    ..Default::default()
                })
                .collect();
            if let Err(e) = insim.send_all(clear).await {
                tracing::warn!("UI attach: failed to clear buttons on shutdown: {e}");
            }
        });
        Ok::<(), UiError>(())
    });

    let handle = tokio::spawn(async move {
        match thread_handle.join() {
            Ok(result) => result,
            Err(_) => {
                tracing::error!("UI thread panicked");
                Err(UiError::RuntimeCreationFailed)
            },
        }
    });

    (ui_handle, handle)
}

#[allow(clippy::type_complexity)]
fn spawn_for<V: View>(
    ucid: ConnectionId,
    global_rx: watch::Receiver<V::GlobalState>,
    insim: &insim::builder::InsimTask,
    active: &mut HashMap<ConnectionId, ActiveViewChannels<V>>,
    outbound: broadcast::Sender<(ConnectionId, V::Message)>,
) {
    // per-view connection props stream (targeted by ucid).
    let (connection_props_tx, connection_props_rx) = watch::channel(V::ConnectionState::default());
    // per-view event stream (external messages + demuxed btc/btt/bfn events).
    let (view_event_tx, view_event_rx) = mpsc::unbounded_channel();

    run_view::<V>(view::RunViewArgs {
        ucid,
        global_props: global_rx,
        connection_props: connection_props_rx,
        view_event_rx,
        insim: insim.clone(),
        outbound,
    });
    let _ = active.insert(
        ucid,
        ActiveViewChannels {
            connection_props_tx,
            view_event_tx,
        },
    );
}

/// Shortcut to make a non-visible container [node::Node]
pub fn container<Msg>() -> node::Node<Msg> {
    node::Node::container()
}

/// Shortcut to make a visible container [node::Node]
pub fn background<Msg>(bstyle: insim::insim::BtnStyle) -> node::Node<Msg> {
    node::Node::background(bstyle)
}

/// Shortcut to make a clickable button [node::Node]
pub fn clickable<Msg>(
    text: impl Into<String>,
    bstyle: insim::insim::BtnStyle,
    msg: Msg,
) -> node::Node<Msg> {
    node::Node::clickable(text, bstyle, msg)
}

/// Shortcut to make a text only (non-clickable) [node::Node]
pub fn text<Msg>(text: impl Into<String>, bstyle: insim::insim::BtnStyle) -> node::Node<Msg> {
    node::Node::text(text, bstyle)
}

/// Shortcut to make a type-in-able [node::Node]
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

/// Shortcut to make an empty [node::Node]
pub fn empty<Msg>() -> node::Node<Msg> {
    node::Node::empty()
}
