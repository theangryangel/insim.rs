use insim::{Packet, identifiers::ConnectionId, insim::BfnType};
use tokio::sync::{mpsc, watch};

use super::Canvas;

pub trait Component<P> {
    type Message: Clone + Send + 'static;
    #[allow(unused)]
    fn update(&mut self, msg: Self::Message) {}
    fn render(&self, props: P) -> super::Node<Self::Message>;
}

/// View
pub trait View:
    Component<(Self::GlobalProps, Self::ConnectionProps)> + Sized + Send + 'static
{
    type GlobalProps: Clone + Send + Sync + Default + 'static;
    type ConnectionProps: Clone + Send + Sync + Default + 'static;

    /// New!
    fn mount(tx: mpsc::UnboundedSender<Self::Message>) -> Self;
}

/// Run the UI
pub(super) fn run_view<V: View>(
    ucid: ConnectionId,
    mut global: watch::Receiver<V::GlobalProps>,
    mut connection: watch::Receiver<V::ConnectionProps>,
    insim: insim::builder::SpawnedHandle,
) -> tokio::task::JoinHandle<()> {
    let (internal_tx, mut internal_rx) = mpsc::unbounded_channel();

    let handle = tokio::spawn(async move {
        let mut root = V::mount(internal_tx);
        let mut packets = insim.subscribe();
        let mut canvas = Canvas::<V>::new(ucid);

        // always draw immediately
        let mut should_render = true;

        loop {
            if should_render {
                let vdom = root.render((
                    global.borrow_and_update().clone(),
                    connection.borrow_and_update().clone(),
                ));
                if let Some(diff) = canvas.paint(&vdom) {
                    dbg!(&diff);

                    // FIXME: no expect
                    insim
                        .send_all(diff.merge())
                        .await
                        .expect("FIXME: send_all failed");
                }
            }

            should_render = tokio::select! {
                Ok(_) = global.changed() => {
                    true
                },

                Ok(_) = connection.changed() => {
                    true
                },

                // internal messages (i.e. clock ticks?)
                Some(msg) = internal_rx.recv() => {
                    root.update(msg);
                    true
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
                                canvas.block();
                                false
                            },
                            BfnType::BtnRequest => {
                                canvas.unblock();
                                true
                            },
                            _ => {
                                false
                            }
                        },
                        _ => {
                            // FIXME: handle Err
                            false
                        }
                    }
                }
            };
        }
    });

    handle
}
