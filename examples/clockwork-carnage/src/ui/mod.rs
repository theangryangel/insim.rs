use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use insim::{
    Packet,
    identifiers::ConnectionId,
    insim::{Bfn, BfnType},
};
use tokio::{
    sync::{Notify, mpsc, watch},
    task::JoinHandle,
};

pub mod node;
pub mod view;

pub use node::*;
pub use view::*;

pub struct UiHandle<V: View> {
    global: watch::Sender<V::GlobalProps>,
    player: mpsc::Sender<(ConnectionId, V::ConnectionProps)>,
    detach: Arc<Notify>,
    _phantom: PhantomData<V>,
}

impl<V: View> UiHandle<V> {
    pub fn update_global_props(&self, value: V::GlobalProps) {
        let _ = self
            .global
            .send(value)
            .expect("FIXME: expected global to work"); // FIXME: check returned value
    }

    pub async fn update_connection_props(&self, ucid: ConnectionId, value: V::ConnectionProps) {
        let _ = self
            .player
            .send((ucid, value))
            .await
            .expect("FIXME: expect connection to work");
    }

    /// Detach the view from all ConnectionIds
    /// This does not need to be manually called, it will automatically be called on Drop.
    pub fn detach(&self) {
        self.detach.notify_waiters();
    }
}

impl<V: View> Drop for UiHandle<V> {
    fn drop(&mut self) {
        self.detach();
    }
}

/// Manager to spawn Ui's for each connection
#[derive(Debug)]
pub struct Ui;

impl Ui {
    pub fn attach<V: View>(
        insim: insim::builder::SpawnedHandle,
        props: V::GlobalProps,
    ) -> UiHandle<V> {
        let (global_tx, global_rx) = watch::channel(props);
        let (player_tx, mut player_rx) = mpsc::channel(100);
        let detach = Arc::new(Notify::new());
        let handle = UiHandle {
            global: global_tx,
            player: player_tx,
            detach: detach.clone(),
            _phantom: PhantomData,
        };

        let _ = tokio::spawn(async move {
            let mut packets = insim.subscribe();
            let mut active: HashMap<
                ConnectionId,
                (
                    watch::Sender<V::ConnectionProps>,
                    JoinHandle<Result<(), insim::Error>>,
                ),
            > = HashMap::new();

            loop {
                tokio::select! {
                    packet = packets.recv() => match packet {
                        Ok(Packet::Ncn(ncn)) => {
                            if active.contains_key(&ncn.ucid) {
                                continue;
                            }

                            // if there's an active view spawn a per-player task because there's a
                            // new player
                            let (connection_tx, connection_rx) = watch::channel(V::ConnectionProps::default());

                            let conn_handle = tokio::spawn(spawn_connection::<V>(
                                ncn.ucid.clone(),
                                global_rx.clone(),
                                connection_rx,
                                insim.clone(),
                            ));

                            let _ = active.insert(ncn.ucid, (
                                connection_tx, conn_handle
                            ));
                        },
                        Ok(Packet::Cnl(cnl)) => {
                            // if there's an active view abort the per-player task because the
                            // player left
                            if let Some((_, handle)) = active.remove(&cnl.ucid) {
                                handle.abort();
                            }
                        },

                        _ => {
                            // FIXME: handle Err
                        }
                    },

                    res = player_rx.recv() => match res {
                        Some((ucid, props)) => {
                            if let Some((entry, _)) = active.get_mut(&ucid) {
                                let _ = entry.send(props);
                            }
                        },
                        None => {
                            // FIXME: log, or something
                            break;
                        }
                    },

                    _ = detach.notified() => {
                        // for all player connections automatically clear all buttons and all tasks
                        // when we loose the UiHandle.
                        let clear: Vec<Bfn> = active.drain().map(|(ucid, (_, handle))| {
                            handle.abort();
                            Bfn { ucid, subt: BfnType::Clear, ..Default::default() }
                        }).collect();
                        // FIXME: no expect
                        insim.send_all(clear).await.expect("FIXME");
                        break;
                    }
                }
            }
        });

        handle
    }
}

async fn spawn_connection<V: View>(
    ucid: ConnectionId,
    mut global: watch::Receiver<V::GlobalProps>,
    mut connection: watch::Receiver<V::ConnectionProps>,
    insim: insim::builder::SpawnedHandle,
) -> Result<(), insim::Error> {
    let mut packet_rx = insim.subscribe();

    loop {
        tokio::select! {
            // Handle button clicks and chat
            Ok(packet) = packet_rx.recv() => {
                let should_render = match packet {
                    Packet::Mso(mso) if mso.ucid == ucid => {
                        todo!(); // handle chat.. this should probably be using Chat...
                    }
                    Packet::Btc(btc) if btc.ucid == ucid => {
                        todo!(); // handle button click
                    },
                    Packet::Bfn(bfn) if bfn.ucid == ucid => match bfn.subt {
                        BfnType::Clear | BfnType::UserClear => {
                            todo!(); // prevent rendering, block
                        },
                        BfnType::BtnRequest => {
                            todo!(); // unblock
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
                    todo!();
                }
            },

            // Handle global changes
            _ = global.changed() => {
                todo!()
            },

            // Handle player changes
            _ = connection.changed() => {
                todo!()
            },
        }
    }
}
