use std::sync::Arc;

use insim::{
    Packet,
    identifiers::{ClickId, ConnectionId},
    insim::BfnType,
};
use tokio::sync::{Notify, broadcast, mpsc, watch};

use super::canvas::Canvas;

/// Reusable Component. Useful for things like self contained clocks, scrollable areas, etc.
/// that require state.
///
/// For stateless components, you can just do something like this:
/// ```rust,ignore
/// use insim_extra::ui::{Node, container, text};
/// use insim::insim::BtnStyle;
/// fn player_badge<Msg>(name: &str, score: u32) -> Node<Msg> {
///     container()
///         .flex_row()
///         .with_child(text(name, BtnStyle::default()))
///         .with_child(text(format!("{score}"), BtnStyle::default()))
/// }
/// ```
pub trait Component {
    /// Messages produced when a user interacts with a clickable / typeable node.
    /// `Send + Sync + Clone + 'static` so the runtime can broadcast each message
    /// to subscribers.
    type Message: Send + Sync + Clone + 'static;
    type Props<'a>;

    #[allow(unused)]
    fn update(&mut self, msg: Self::Message) {}
    fn render(&self, props: Self::Props<'_>) -> super::Node<Self::Message>;
}

/// A root view that a [`Ui`](super::Ui) mounts once per connection.
///
/// A `View` is a [`Component`] (it renders and updates the same way) refined with
/// the state the `Ui` feeds it: `Global` is the UI's global state, broadcast to
/// every view; `Connection` is the per-connection state pushed to one view. The
/// `Ui` re-renders a view when either changes, assembling the component's
/// [`Props`](Component::Props) via [`props`](View::props).
///
/// A reusable building block that is composed inside a view (rather than mounted
/// by the `Ui`) implements only [`Component`] - it takes whatever `Props` its
/// parent passes and is never fed global / per-connection state.
pub trait View: Component {
    /// The UI's global state, broadcast to every mounted view.
    type Global: Send + Sync + 'static;
    /// Per-connection state pushed to a single view.
    type Connection: Clone + Send + Sync + Default + 'static;

    /// Assemble the component's render [`Props`](Component::Props) from the
    /// current global and per-connection state. For the common
    /// `Props<'a> = (&'a Global, &'a Connection)` this is just `(global, connection)`.
    fn props<'a>(global: &'a Self::Global, connection: &'a Self::Connection) -> Self::Props<'a>;
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
pub(super) enum ViewInput<M> {
    Message(M),
    Click { clickid: ClickId },
    TypeIn { clickid: ClickId, text: String },
    Bfn { subt: BfnType },
}

pub(super) struct RunViewArgs<V>
where
    V: View,
{
    pub ucid: ConnectionId,
    pub root: V,
    pub invalidation_notify: Arc<Notify>,
    pub global_props: watch::Receiver<Arc<V::Global>>,
    pub connection_props: watch::Receiver<V::Connection>,
    pub view_event_rx: mpsc::UnboundedReceiver<ViewInput<V::Message>>,
    pub outgoing_tx: mpsc::UnboundedSender<Packet>,
    pub message_tx: broadcast::Sender<V::Message>,
}

/// Run the UI on a LocalSet (does not require Send)
pub(super) fn run_view<V>(args: RunViewArgs<V>)
where
    V: View + 'static,
{
    let RunViewArgs {
        ucid,
        root,
        invalidation_notify,
        mut global_props,
        mut connection_props,
        mut view_event_rx,
        outgoing_tx,
        message_tx,
    } = args;

    #[allow(clippy::let_underscore_future)]
    let _ = tokio::task::spawn_local(async move {
        let mut root = root;
        let mut canvas = Canvas::<V::Message>::new(ucid);
        let mut blocked = false; // user cleared the buttons, do not redraw unless requested
        let mut view_event_rx_closed = false;

        // always draw immediately
        let mut should_render = true;

        loop {
            if should_render && !blocked {
                let global = Arc::clone(&*global_props.borrow_and_update());
                let player = connection_props.borrow_and_update();
                let vdom = root.render(V::props(&global, &player));
                if let Some(diff) = canvas.reconcile(vdom) {
                    let mut any_err = false;
                    for packet in diff.merge() {
                        if outgoing_tx.send(packet).is_err() {
                            tracing::error!("Failed to send UI packet: receiver closed");
                            any_err = true;
                            break;
                        }
                    }
                    if any_err {
                        break;
                    }
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

                // per-view events (external messages + demuxed UI input)
                event = view_event_rx.recv(), if !view_event_rx_closed => {
                    match event {
                        Some(ViewInput::Message(msg)) => {
                            root.update(msg);
                            true
                        },
                        Some(ViewInput::Click { clickid }) => {
                            if let Some(msg) = canvas.translate_clickid(&clickid) {
                                let _ = message_tx.send(msg.clone());
                                root.update(msg);
                                true
                            } else {
                                false
                            }
                        },
                        Some(ViewInput::TypeIn { clickid, text }) => {
                            if let Some(msg) = canvas.translate_typein_clickid(&clickid, text) {
                                let _ = message_tx.send(msg.clone());
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
                            view_event_rx_closed = true;
                            false
                        }
                    }
                },

                // in-view invalidation requests (e.g. timers/marquees)
                _ = invalidation_notify.notified() => {
                    true
                }
            };
        }

        tracing::debug!("Child UI view shutdown");
    });
}
