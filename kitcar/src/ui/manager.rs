use std::collections::HashMap;

use insim::{
    identifiers::ConnectionId,
    insim::{BfnType, TinyType},
    Packet, WithRequestId,
};
use tokio::{sync::watch, task::JoinHandle};

use crate::ui::{ClickIdPool, Component, Runtime};

/// Manager to implement Ui
#[derive(Debug)]
pub struct Manager;

impl Manager {
    pub fn spawn<C: Component>(
        signals: watch::Receiver<C::Props>,
        insim: insim::builder::SpawnedHandle,
    ) -> JoinHandle<insim::Result<()>> {
        tokio::spawn(async move {
            let mut packet_rx = insim.subscribe();
            let mut active = HashMap::new();

            let _ = insim.send(TinyType::Ncn.with_request_id(1)).await?;

            while let Ok(packet) = packet_rx.recv().await {
                match packet {
                    Packet::Ncn(ncn) => {
                        let _ = active.entry(ncn.ucid).or_insert_with(|| {
                            Self::spawn_player_ui::<C>(ncn.ucid, signals.clone(), insim.clone())
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

            // FIXME: masking the error if one occurs
            Ok(())
        })
    }

    fn spawn_player_ui<C: Component>(
        ucid: ConnectionId,
        mut signals: watch::Receiver<C::Props>,
        insim: insim::builder::SpawnedHandle,
    ) -> JoinHandle<insim::Result<()>> {
        tokio::spawn(async move {
            let mut runtime = Runtime::new(ClickIdPool::new());

            // Initial render
            let props = signals.borrow().clone();
            if let Some(diff) = runtime.render::<C>(props, &ucid) {
                insim.send_all(diff.into_merged()).await?;
            }

            let mut packet_rx = insim.subscribe();

            loop {
                tokio::select! {
                    // Handle button clicks
                    Ok(packet) = packet_rx.recv() => {
                        let render_result = match packet {
                            Packet::Btc(btc) => {
                                if btc.ucid == ucid {
                                    runtime.on_click(&btc.clickid);
                                    Some(true)
                                } else {
                                    None
                                }
                            },
                            Packet::Bfn(bfn) => {
                                if bfn.ucid != ucid {
                                    None
                                }
                                else if matches!(bfn.subt, BfnType::Clear | BfnType::UserClear) {
                                    runtime.block();
                                    None
                                }
                                else if matches!(bfn.subt, BfnType::BtnRequest) {
                                    runtime.unblock();
                                    Some(true)
                                } else {
                                    None
                                }
                            },

                            _ => {
                                None
                            }
                        };

                        if render_result.unwrap_or(false) {
                            let props = signals.borrow().clone();
                            if let Some(diff) = runtime.render::<C>(props, &ucid) {
                                insim.send_all(diff.into_merged()).await?;
                            }
                        }
                    },

                    // Handle signal changes
                    _ = signals.changed() => {
                        let props = signals.borrow_and_update().clone();
                        if let Some(diff) = runtime.render::<C>(props, &ucid) {
                            insim.send_all(diff.into_merged()).await?;
                        }
                    }
                }
            }
        })
    }
}
