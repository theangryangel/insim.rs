use std::{collections::HashMap, marker::PhantomData};

use insim::{
    Packet,
    identifiers::ConnectionId,
    insim::{Bfn, BfnType},
};
use kitcar::presence::PresenceHandle;
use tokio::{
    sync::{mpsc, watch},
    task::LocalSet,
};

mod canvas;
pub mod node;
pub mod view;

pub use node::*;
pub use view::*;

/// Ui handle. Create using [attach]. When dropped all insim buttons will be automatically removed.
/// Intended for multi-player/multi-connection UIs
pub struct Ui<V: View> {
    global: watch::Sender<V::GlobalProps>,
    connection: mpsc::Sender<(ConnectionId, V::ConnectionProps)>,
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
}

/// Manager to spawn Ui's for each connection
/// Dropping the returned Ui handle will result in the UI being cleared
///
/// All UI tasks run on a LocalSet, so View implementations don't need to be Send to accomodate
/// taffy
pub fn attach<V: View>(
    insim: insim::builder::SpawnedHandle,
    presence: PresenceHandle,
    props: V::GlobalProps,
) -> Ui<V> {
    let (global_tx, global_rx) = watch::channel(props);
    let (player_tx, mut player_rx) = mpsc::channel(100);
    let handle = Ui {
        global: global_tx,
        connection: player_tx,
        _phantom: PhantomData,
    };

    // XXX: We run on our own thread because we need to use LocalSet until Taffy Style is Send.
    // https://github.com/DioxusLabs/taffy/issues/823
    let _ = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create UI runtime");

        let local = LocalSet::new();
        local.block_on(&rt, async move {
            let mut packets = insim.subscribe();
            let mut active: HashMap<ConnectionId, watch::Sender<V::ConnectionProps>> =
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
                            // player left, remove their props sender
                            let _ = active.remove(&cnl.ucid);
                        },

                        _ => {
                            // FIXME: handle Err
                        }
                    },

                    res = player_rx.recv() => match res {
                        Some((ucid, props)) => {
                            if let Some(entry) = active.get_mut(&ucid) {
                                let _ = entry.send(props);
                            }
                        },
                        None => {
                            // FIXME: log, or something. we've probably just dropped the ui handle
                            break;
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
    });

    handle
}

fn spawn_for<V: View>(
    ucid: ConnectionId,
    global_rx: &watch::Receiver<V::GlobalProps>,
    insim: &insim::builder::SpawnedHandle,
    active: &mut HashMap<ConnectionId, watch::Sender<V::ConnectionProps>>,
) {
    let (connection_tx, connection_rx) = watch::channel(V::ConnectionProps::default());

    run_view::<V>(
        ucid.clone(),
        global_rx.clone(),
        connection_rx,
        insim.clone(),
    );
    let _ = active.insert(ucid, connection_tx);
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
