use insim::{Packet, identifiers::ConnectionId, insim::BfnType};
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
    type Message: Clone + 'static;
    type Props;

    #[allow(unused)]
    fn update(&mut self, msg: Self::Message) {}
    fn render(&self, props: Self::Props) -> super::Node<Self::Message>;
}

/// View - think of it as the root [Component].
pub trait View: Sized + 'static {
    type GlobalProps: Clone + Send + Sync + Default + 'static;
    type ConnectionProps: Clone + Send + Sync + Default + 'static;
    type Message: Clone + 'static;

    /// New!
    fn mount(tx: mpsc::UnboundedSender<Self::Message>) -> Self;

    #[allow(unused)]
    fn update(&mut self, msg: Self::Message) {}
    fn render(
        &self,
        global_props: Self::GlobalProps,
        connection_props: Self::ConnectionProps,
    ) -> super::Node<Self::Message>;
}

/// Run the UI on a LocalSet (does not require Send)
pub(super) fn run_view<V: View>(
    ucid: ConnectionId,
    mut global: watch::Receiver<V::GlobalProps>,
    mut connection: watch::Receiver<V::ConnectionProps>,
    insim: insim::builder::SpawnedHandle,
) {
    let (internal_tx, mut internal_rx) = mpsc::unbounded_channel();

    #[allow(clippy::let_underscore_future)]
    let _ = tokio::task::spawn_local(async move {
        let mut root = V::mount(internal_tx);
        let mut packets = insim.subscribe();
        let mut canvas = Canvas::<V>::new(ucid);
        let mut blocked = false; // user cleared the buttons, do not redraw unless requested
        let mut internal_closed = false;

        // always draw immediately
        let mut should_render = true;

        loop {
            if should_render && !blocked {
                let vdom = root.render(
                    global.borrow_and_update().clone(),
                    connection.borrow_and_update().clone(),
                );
                if let Some(diff) = canvas.reconcile(vdom) {
                    // FIXME: no expect
                    insim
                        .send_all(diff.merge())
                        .await
                        .expect("FIXME: send_all failed");
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

                // user input (click ids)
                packet = packets.recv() => {
                    match packet {
                        Ok(Packet::Btc(btc)) if btc.ucid == ucid => {
                            if let Some(msg) = canvas.translate_clickid(&btc.clickid) {
                                root.update(msg);
                                true
                            } else {
                                false
                            }
                        },
                        Ok(Packet::Bfn(bfn)) if bfn.ucid == ucid => match bfn.subt {
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
                        Err(e) => {
                            tracing::error!("Failed to receive packets from insim: {e}");
                            false
                        }
                        _ => {
                            // FIXME: handle Err
                            false
                        }
                    }
                }
            };
        }

        tracing::error!("Child shutdown");
    });
}
