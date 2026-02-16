use insim::{
    identifiers::{ClickId, ConnectionId},
    insim::BfnType,
};
use std::sync::Arc;

use tokio::sync::{Notify, mpsc, watch};

use super::canvas::Canvas;

/// Reusable Component. Useful for things like self contained clocks, scrollable areas, etc.
/// that require state.
/// For stateless components, you can just do something like this:
/// ```rust
/// use kitcar::ui::{Node, container, text};
/// use insim::insim::BtnStyle;
/// fn player_badge<Msg>(name: &str, score: u32) -> Node<Msg> {
///     container()
///         .flex_row()
///         .with_child(text(name, BtnStyle::default()))
///         .with_child(text(format!("{score}"), BtnStyle::default()))
/// }
// ```
pub trait Component {
    type Message: Send + Clone + 'static;
    type Props;

    #[allow(unused)]
    fn update(&mut self, msg: Self::Message) {}
    fn render(&self, props: Self::Props) -> super::Node<Self::Message>;
}

/// Root multiplayer UI type.
pub trait View: Component + Sized + 'static {
    type GlobalState: Clone + Send + Sync + Default + 'static;
    type ConnectionState: Clone + Send + Sync + Default + 'static;

    /// Called once when a per-connection view instance is spawned.
    ///
    /// Use [`InvalidateHandle`] for time-based components (marquees, clocks, stopwatches)
    /// to request a re-render without routing synthetic messages through `update`.
    fn mount(invalidator: InvalidateHandle) -> Self;
    fn compose(global: Self::GlobalState, connection: Self::ConnectionState) -> Self::Props;
}

/// Handle to request a redraw of the current view instance.
#[derive(Clone, Debug)]
pub struct InvalidateHandle {
    notify: Arc<Notify>,
}

impl InvalidateHandle {
    pub(super) fn new(notify: Arc<Notify>) -> Self {
        Self { notify }
    }

    /// Request a re-render of the current view instance.
    pub fn invalidate(&self) {
        self.notify.notify_one();
    }
}

#[derive(Debug)]
pub(super) enum ViewInput {
    Click { clickid: ClickId },
    TypeIn { clickid: ClickId, text: String },
    Bfn { subt: BfnType },
}

pub(super) struct RunViewArgs<V: View> {
    pub ucid: ConnectionId,
    // global props stream shared by all connected players.
    pub global_props: watch::Receiver<V::GlobalState>,
    // per-connection props stream for this specific `ucid`.
    pub connection_props: watch::Receiver<V::ConnectionState>,
    // receiver of per-view messages from external `Ui::update`/`Ui::sender`.
    pub view_msg_rx: mpsc::UnboundedReceiver<V::Message>,
    // demuxed ui input events for this `ucid`.
    pub ui_input_rx: mpsc::UnboundedReceiver<ViewInput>,
    pub insim: insim::builder::InsimTask,
    pub outbound: tokio::sync::broadcast::Sender<(ConnectionId, V::Message)>,
}

/// Run the UI on a LocalSet (does not require Send)
pub(super) fn run_view<V: View>(args: RunViewArgs<V>) {
    let RunViewArgs {
        ucid,
        mut global_props,
        mut connection_props,
        mut view_msg_rx,
        mut ui_input_rx,
        insim,
        outbound,
    } = args;

    #[allow(clippy::let_underscore_future)]
    let _ = tokio::task::spawn_local(async move {
        let invalidation_notify = Arc::new(Notify::new());
        let mut root = V::mount(InvalidateHandle::new(invalidation_notify.clone()));
        let mut canvas = Canvas::<V::Message>::new(ucid);
        let mut blocked = false; // user cleared the buttons, do not redraw unless requested
        let mut view_msg_rx_closed = false;
        let mut ui_input_rx_closed = false;

        // always draw immediately
        let mut should_render = true;

        loop {
            if should_render && !blocked {
                let vdom = root.render(V::compose(
                    global_props.borrow_and_update().clone(),
                    connection_props.borrow_and_update().clone(),
                ));
                if let Some(diff) = canvas.reconcile(vdom)
                    && let Err(e) = insim.send_all(diff.merge()).await
                {
                    tracing::error!("Failed to send UI diff packets: {e}");
                    break;
                }
            }

            should_render = tokio::select! {
                res = global_props.changed() => {
                    if res.is_err() {
                        break;
                    }
                    true
                },

                res = connection_props.changed() => {
                    if res.is_err() {
                        break;
                    }
                    true
                },

                // view messages (component self-messages and external Ui::update messages)
                msg = view_msg_rx.recv(), if !view_msg_rx_closed => {
                    match msg {
                        Some(msg) => {
                            root.update(msg);
                            true
                        },
                        None => {
                            view_msg_rx_closed = true;
                            false
                        }
                    }
                },

                // in-view invalidation requests (e.g. timers/marquees)
                _ = invalidation_notify.notified() => {
                    true
                },

                // user input from demux (click ids)
                packet = ui_input_rx.recv(), if !ui_input_rx_closed => {
                    match packet {
                        Some(ViewInput::Click { clickid }) => {
                            if let Some(msg) = canvas.translate_clickid(&clickid) {
                                let _ = outbound.send((ucid, msg.clone()));
                                root.update(msg);
                                true
                            } else {
                                false
                            }
                        },
                        Some(ViewInput::TypeIn { clickid, text }) => {
                            if let Some(msg) = canvas.translate_typein_clickid(&clickid, text) {
                                let _ = outbound.send((ucid, msg.clone()));
                                root.update(msg);
                                true
                            } else {
                                false
                            }
                        }
                        Some(ViewInput::Bfn { subt }) => match subt {
                            BfnType::Clear | BfnType::UserClear => {
                                blocked = true;
                                canvas.clear();
                                false
                            },
                            BfnType::BtnRequest => {
                                blocked = false;
                                true
                            },
                            _ => {
                                false
                            }
                        },
                        None => {
                            ui_input_rx_closed = true;
                            false
                        }
                    }
                }
            };
        }

        tracing::debug!("Child UI view shutdown");
    });
}
