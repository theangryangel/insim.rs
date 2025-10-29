use std::collections::HashMap;

use insim::{Packet, identifiers::ConnectionId, insim::BfnType};
use tokio::{sync::watch, task::JoinHandle};

use crate::ui::{ClickIdPool, Component, Runtime};

#[derive(Debug, Clone)]
pub struct ManagerHandle<C: Component> {
    props_tx: watch::Sender<C::Props>,
}

impl<C: Component> ManagerHandle<C> {
    pub fn update(&self, props: C::Props) {
        let _ = self.props_tx.send(props); // FIXME
    }
}

/// Manager to spawn Ui's for each connection
#[derive(Debug)]
pub struct Manager;

impl Manager {
    pub fn spawn<C: Component>(
        insim: insim::builder::SpawnedHandle,
        props: C::Props,
    ) -> (ManagerHandle<C>, std::thread::JoinHandle<insim::Result<()>>) {
        let (props_tx, props_rx) = watch::channel(props);

        let handle = ManagerHandle { props_tx };

        let join_handle = std::thread::spawn(move || {
            // We spawn the UI as it's own thread, so that we can use a tokio LocalSet.
            // So that we can use Rc rather than Mutex to handle our component state
            // and to avoid a whole load of Send issues if the user attempts to use !Send values
            // within their state

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let local_set = tokio::task::LocalSet::new();
            local_set.block_on(&rt, async move {
                let mut packet_rx = insim.subscribe();
                let mut active = HashMap::new();

                while let Ok(packet) = packet_rx.recv().await {
                    match packet {
                        Packet::Ncn(ncn) => {
                            let _clippy = active.entry(ncn.ucid).or_insert_with(|| {
                                Self::spawn_player_ui::<C>(
                                    ncn.ucid,
                                    props_rx.clone(),
                                    insim.clone(),
                                )
                            });
                        },
                        Packet::Cnl(cnl) => {
                            if let Some(handle) = active.remove(&cnl.ucid) {
                                handle.abort();
                            }
                        },
                        _ => {},
                    }
                }
            });

            // FIXME: masking the error if one occurs
            Ok(())
        });

        (handle, join_handle)
    }

    fn spawn_player_ui<C: Component>(
        ucid: ConnectionId,
        mut signals: watch::Receiver<C::Props>,
        insim: insim::builder::SpawnedHandle,
    ) -> JoinHandle<insim::Result<()>> {
        tokio::task::spawn_local(async move {
            let mut runtime = Runtime::<C>::new(ClickIdPool::new(), ucid);

            // Initial render
            runtime
                .render_diff_send(signals.borrow().clone(), &insim)
                .await?;

            let mut packet_rx = insim.subscribe();

            loop {
                tokio::select! {
                    // Handle button clicks and chat
                    Ok(packet) = packet_rx.recv() => {
                        let should_render = match packet {
                            Packet::Mso(mso) if mso.ucid == ucid => {
                                runtime.on_chat(&mso)
                            }
                            Packet::Btc(btc) if btc.ucid == ucid => {
                                runtime.on_click(&btc.clickid);
                                true
                            },
                            Packet::Bfn(bfn) if bfn.ucid == ucid => match bfn.subt {
                                BfnType::Clear | BfnType::UserClear => {
                                    runtime.block();
                                    false
                                },
                                BfnType::BtnRequest => {
                                    runtime.unblock();
                                    true
                                },
                                _ => {
                                    false
                                }
                            },
                            _ => {
                                false
                            }
                        };

                        if should_render {
                            runtime.render_diff_send(signals.borrow().clone(), &insim).await?;
                        }
                    },

                    // Handle signal changes
                    _ = signals.changed() => {
                        runtime.render_diff_send(signals.borrow().clone(), &insim).await?;
                    }
                }
            }
        })
    }
}
