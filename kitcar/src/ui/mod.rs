use std::collections::HashMap;

use insim::{
    Packet,
    identifiers::ConnectionId,
    insim::{Bfn, BfnType},
};
use tokio::{
    sync::{broadcast, mpsc, watch},
    task::{JoinHandle, LocalSet},
};

use crate::presence::Presence;

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

/// Ui handle. Create using [attach]. When dropped all insim buttons will be automatically removed.
/// Intended for multi-player/multi-connection UIs
#[derive(Debug)]
pub struct Ui<M: Clone + Send + 'static, G, C> {
    global: watch::Sender<G>,
    connection: mpsc::Sender<(ConnectionId, C)>,
    message: mpsc::Sender<(ConnectionId, M)>,
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
        self.global
            .send(value)
            .expect("FIXME: expect global to work");
    }

    /// Update the state for a specific connection, triggering a re-render for that player.
    /// Player state is per-player state, useful for player-specific UI elements.
    pub async fn set_player_state(&self, ucid: ConnectionId, value: C) {
        self.connection
            .send((ucid, value))
            .await
            .expect("FIXME: expect connection to work");
    }

    /// Get a clonable sender for injecting messages into UI components.
    /// This sender can be moved into spawned tasks without affecting UI lifecycle.
    /// Sending will fail once the [`Ui`] handle is dropped.
    ///
    /// Cloning this sender does not keep the UI alive—the UI lifecycle is controlled
    /// by the main [`Ui`] handle. When [`Ui`] is dropped, the UI runtime shuts down
    /// and subsequent sends will return `Err`.
    ///
    /// You likely want to use the listen function.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let (ui, _handle) = attach::<MyView>(insim, presence, props);
    /// let sender = ui.sender();
    /// let mut chat_rx = chat.subscribe();
    ///
    /// tokio::spawn(async move {
    ///     while let Ok((cmd, ucid)) = chat_rx.recv().await {
    ///         if let Some(msg) = map_to_view_msg(cmd) {
    ///             if sender.send((ucid, msg)).await.is_err() {
    ///                 break; // UI dropped, exit loop
    ///             }
    ///         }
    ///     }
    /// });
    ///
    /// // ui can still be used here
    /// ui.set_global_state(new_state);
    /// ```
    pub fn sender(&self) -> mpsc::Sender<(ConnectionId, M)> {
        self.message.clone()
    }

    /// Subscribe to messages produced by user UI interactions (e.g. button click / type-in).
    pub fn subscribe(&self) -> broadcast::Receiver<(ConnectionId, M)> {
        self.outbound.subscribe()
    }

    /// Utility function to reduce boilerplate when injecting messages from external sources,
    /// usually chat.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let _chat_task = ui.update_from_broadcast(self.chat.subscribe(), |msg, _ucid| {
    ///     matches!(msg, chat::ChatMsg::Help)
    ///         .then_some(ClockworkLobbyMessage::Help(HelpDialogMsg::Show))
    /// });
    /// ```
    pub fn update_from_broadcast<F, N>(
        &self,
        mut rx: broadcast::Receiver<(N, ConnectionId)>,
        mut filter_map: F,
    ) -> JoinHandle<()>
    where
        N: Clone + Send + 'static,
        F: FnMut(N, ConnectionId) -> Option<M> + Send + 'static,
    {
        let tx = self.sender();
        tokio::spawn(async move {
            while let Ok((msg, ucid)) = rx.recv().await {
                if let Some(ui_msg) = filter_map(msg, ucid)
                    && tx.send((ucid, ui_msg)).await.is_err()
                {
                    break;
                }
            }
        })
    }

    /// Inject message into UI for a given ucid.
    pub async fn update(&self, ucid: ConnectionId, msg: M) {
        let _ = self.message.send((ucid, msg)).await;
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
///     fn mount(_tx: mpsc::UnboundedSender<Self::Message>) -> Self {
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
/// let (ui, handle) = attach::<MyView>(insim, presence, GameState::default());
///
/// // Update global state (re-renders for all players)
/// ui.set_global_state(GameState { score: 100 });
///
/// // Update per-player state
/// ui.set_player_state(player_ucid, PlayerState { ready: true }).await;
/// ```
pub fn attach<V>(
    insim: insim::builder::InsimTask,
    presence: Presence,
    props: V::GlobalState,
) -> (
    Ui<V::Message, V::GlobalState, V::ConnectionState>,
    JoinHandle<Result<(), UiError>>,
)
where
    V: View,
{
    let (global_tx, _global_rx) = watch::channel(props);
    let (player_tx, mut player_rx) = mpsc::channel(100);
    let (message_tx, mut message_rx) = mpsc::channel::<(ConnectionId, V::Message)>(100);
    let (outbound_tx, _outbound_rx) = broadcast::channel::<(ConnectionId, V::Message)>(100);
    let ui_handle = Ui {
        global: global_tx.clone(),
        connection: player_tx,
        message: message_tx,
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
            #[allow(clippy::type_complexity)]
            let mut active: HashMap<
                ConnectionId,
                (
                    watch::Sender<V::ConnectionState>,
                    mpsc::UnboundedSender<V::Message>,
                ),
            > = HashMap::new();

            // FIXME: expect
            for existing in presence.connections().await.expect("FIXME") {
                spawn_for::<V>(
                    existing.ucid,
                    global_tx.subscribe(),
                    &insim,
                    &mut active,
                    outbound_tx.clone(),
                );
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

                        _ => {
                            // FIXME: handle Err
                        }
                    },

                    res = player_rx.recv() => match res {
                        Some((ucid, props)) => {
                            if let Some((props_tx, _)) = active.get_mut(&ucid) {
                                let _ = props_tx.send(props);
                            }
                        },
                        None => {
                            // FIXME: log, or something. we've probably just dropped the ui handle
                            break;
                        }
                    },

                    // Message channel is separate from lifecycle—UiSender clones keep this
                    // channel open, but we shut down based on player_rx (connection props).
                    // Once Ui is dropped and we exit this loop, message_rx is dropped and
                    // UiSender::send() will return Err.
                    res = message_rx.recv() => match res {
                        Some((ucid, msg)) => {
                            if let Some((_, msg_tx)) = active.get(&ucid) {
                                let _ = msg_tx.send(msg);
                            }
                        },
                        None => {
                            // All senders dropped (Ui + all UiSender clones), but we don't
                            // shut down here—lifecycle is tied to player_rx.
                        }
                    },
                }
            }

            // for all player connections automatically clear all buttons
            // when we loose the UiHandle.
            // this should happen when we loose the player_rx receiver.
            let clear: Vec<Bfn> = active
                .drain()
                .map(|(ucid, _)| Bfn {
                    ucid,
                    subt: BfnType::Clear,
                    ..Default::default()
                })
                .collect();
            // FIXME: no expect
            insim.send_all(clear).await.expect("FIXME");
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
    active: &mut HashMap<
        ConnectionId,
        (
            watch::Sender<V::ConnectionState>,
            mpsc::UnboundedSender<V::Message>,
        ),
    >,
    outbound: broadcast::Sender<(ConnectionId, V::Message)>,
) {
    let (connection_tx, connection_rx) = watch::channel(V::ConnectionState::default());
    let (internal_tx, internal_rx) = mpsc::unbounded_channel();

    run_view::<V>(
        ucid,
        global_rx,
        connection_rx,
        internal_tx.clone(),
        internal_rx,
        insim.clone(),
        outbound,
    );
    let _ = active.insert(ucid, (connection_tx, internal_tx));
}

/// Shortcut to make a container [node::Node]
pub fn container<Msg>() -> node::Node<Msg> {
    node::Node::container()
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
