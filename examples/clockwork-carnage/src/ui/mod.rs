use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use insim::{
    Packet,
    identifiers::ConnectionId,
    insim::{Bfn, BfnType},
};
use kitcar::presence::PresenceHandle;
use tokio::{
    sync::{Notify, mpsc, watch},
    task::JoinHandle,
};

pub mod canvas;
pub mod node;
pub mod view;

pub use canvas::*;
pub use node::*;
pub use view::*;

pub struct Ui<V: View> {
    global: watch::Sender<V::GlobalProps>,
    connection: mpsc::Sender<(ConnectionId, V::ConnectionProps)>,
    detach: Arc<Notify>,
    _phantom: PhantomData<V>,
}

impl<V: View> Ui<V> {
    pub fn update_global_props(&self, value: V::GlobalProps) {
        let _ = self
            .global
            .send(value)
            .expect("FIXME: expected global to work"); // FIXME: check returned value
    }

    pub async fn update_connection_props(&self, ucid: ConnectionId, value: V::ConnectionProps) {
        let _ = self
            .connection
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

impl<V: View> Drop for Ui<V> {
    fn drop(&mut self) {
        self.detach();
    }
}

/// Manager to spawn Ui's for each connection
/// Dropping the returned Ui handle will result in the UI being cleared
pub fn attach<V: View>(
    insim: insim::builder::SpawnedHandle,
    presence: PresenceHandle,
    props: V::GlobalProps,
) -> Ui<V> {
    let (global_tx, global_rx) = watch::channel(props);
    let (player_tx, mut player_rx) = mpsc::channel(100);
    let detach = Arc::new(Notify::new());
    let handle = Ui {
        global: global_tx,
        connection: player_tx,
        detach: detach.clone(),
        _phantom: PhantomData,
    };

    let _ = tokio::spawn(async move {
        let mut packets = insim.subscribe();
        let mut active: HashMap<ConnectionId, (watch::Sender<V::ConnectionProps>, JoinHandle<()>)> =
            HashMap::new();

        // FIXME: expect
        for existing in presence.connections().await.expect("FIXME") {
            spawn_for::<V>(existing.ucid, &global_rx, &insim, &mut active);
        }

        loop {
            tokio::select! {
                packet = packets.recv() => match packet {
                    Ok(Packet::Ncn(ncn)) => {
                        if active.contains_key(&ncn.ucid) {
                            continue;
                        }

                        spawn_for::<V>(ncn.ucid, &global_rx, &insim, &mut active);
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

fn spawn_for<V: View>(
    ucid: ConnectionId,
    global_rx: &watch::Receiver<V::GlobalProps>,
    insim: &insim::builder::SpawnedHandle,
    active: &mut HashMap<ConnectionId, (watch::Sender<V::ConnectionProps>, JoinHandle<()>)>,
) {
    let (connection_tx, connection_rx) = watch::channel(V::ConnectionProps::default());

    let conn_handle = run_view::<V>(
        ucid.clone(),
        global_rx.clone(),
        connection_rx,
        insim.clone(),
    );
    let _ = active.insert(ucid, (connection_tx, conn_handle));
}
