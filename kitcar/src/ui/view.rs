use insim::{
    identifiers::{ClickId, ConnectionId},
    insim::BfnType,
};
use tokio::sync::{mpsc, watch};

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

    fn mount(tx: mpsc::UnboundedSender<Self::Message>) -> Self;
    fn compose(global: Self::GlobalState, connection: Self::ConnectionState) -> Self::Props;
}

#[derive(Debug)]
pub(super) enum ViewInput {
    Click { clickid: ClickId },
    TypeIn { clickid: ClickId, text: String },
    Bfn { subt: BfnType },
}

pub(super) struct RunViewArgs<V: View> {
    pub ucid: ConnectionId,
    pub global: watch::Receiver<V::GlobalState>,
    pub connection: watch::Receiver<V::ConnectionState>,
    pub internal_tx: mpsc::UnboundedSender<V::Message>,
    pub internal_rx: mpsc::UnboundedReceiver<V::Message>,
    pub input_rx: mpsc::UnboundedReceiver<ViewInput>,
    pub insim: insim::builder::InsimTask,
    pub outbound: tokio::sync::broadcast::Sender<(ConnectionId, V::Message)>,
}

/// Run the UI on a LocalSet (does not require Send)
pub(super) fn run_view<V: View>(args: RunViewArgs<V>) {
    let RunViewArgs {
        ucid,
        mut global,
        mut connection,
        internal_tx,
        mut internal_rx,
        mut input_rx,
        insim,
        outbound,
    } = args;

    #[allow(clippy::let_underscore_future)]
    let _ = tokio::task::spawn_local(async move {
        let mut root = V::mount(internal_tx);
        let mut canvas = Canvas::<V::Message>::new(ucid);
        let mut blocked = false; // user cleared the buttons, do not redraw unless requested
        let mut internal_closed = false;
        let mut input_closed = false;

        // always draw immediately
        let mut should_render = true;

        loop {
            if should_render && !blocked {
                let vdom = root.render(V::compose(
                    global.borrow_and_update().clone(),
                    connection.borrow_and_update().clone(),
                ));
                if let Some(diff) = canvas.reconcile(vdom)
                    && let Err(e) = insim.send_all(diff.merge()).await
                {
                    tracing::error!("Failed to send UI diff packets: {e}");
                    break;
                }
            }

            should_render = tokio::select! {
                res = global.changed() => {
                    if res.is_err() {
                        break;
                    }
                    true
                },

                res = connection.changed() => {
                    if res.is_err() {
                        break;
                    }
                    true
                },

                // internal messages (i.e. clock ticks?)
                msg = internal_rx.recv(), if !internal_closed => {
                    match msg {
                        Some(msg) => {
                            root.update(msg);
                            true
                        },
                        None => {
                            internal_closed = true;
                            false
                        }
                    }
                },

                // user input from demux (click ids)
                packet = input_rx.recv(), if !input_closed => {
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
                            input_closed = true;
                            false
                        }
                    }
                }
            };
        }

        tracing::debug!("Child UI view shutdown");
    });
}
