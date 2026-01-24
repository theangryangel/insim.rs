use std::{collections::HashMap, marker::PhantomData};

use insim::{
    Packet,
    identifiers::ConnectionId,
    insim::{Bfn, BfnType},
};
use tokio::{
    sync::{mpsc, watch},
    task::{JoinHandle, LocalSet},
};

use crate::presence::Presence;

mod canvas;
pub mod id_pool;
mod node;
mod view;

pub use node::*;
pub use view::*;

#[derive(Debug, thiserror::Error)]
/// UiError
pub enum UiError {
    /// Failed to create UI runtime
    #[error("Failed to create UI runtime")]
    RuntimeCreationFailed,
}

/// Ui handle. Create using [attach]. When dropped all insim buttons will be automatically removed.
/// Intended for multi-player/multi-connection UIs
#[derive(Debug)]
pub struct Ui<V: View> {
    global: watch::Sender<V::GlobalProps>,
    connection: mpsc::Sender<(ConnectionId, V::ConnectionProps)>,
    _phantom: PhantomData<V>,
}

impl<V: View> Ui<V> {
    pub fn update_global_props(&self, value: V::GlobalProps) {
        self.global
            .send(value)
            .expect("FIXME: expect global to work");
    }

    pub async fn update_connection_props(&self, ucid: ConnectionId, value: V::ConnectionProps) {
        self.connection
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
    insim: insim::builder::InsimTask,
    presence: Presence,
    props: V::GlobalProps,
) -> (Ui<V>, JoinHandle<Result<(), UiError>>) {
    let (global_tx, _global_rx) = watch::channel(props);
    let (player_tx, mut player_rx) = mpsc::channel(100);
    let ui_handle = Ui {
        global: global_tx.clone(),
        connection: player_tx,
        _phantom: PhantomData,
    };

    drop(_global_rx);

    // XXX: We run on our own thread because we need to use LocalSet until Taffy Style is Send.
    // https://github.com/DioxusLabs/taffy/issues/823
    let thread_handle = std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                tracing::error!("Failed to create UI runtime: {}", e);
                return Err::<(), UiError>(UiError::RuntimeCreationFailed);
            },
        };

        let local = LocalSet::new();
        local.block_on(&rt, async move {
            let mut packets = insim.subscribe();
            let mut active: HashMap<ConnectionId, watch::Sender<V::ConnectionProps>> =
                HashMap::new();

            // FIXME: expect
            for existing in presence.connections().await.expect("FIXME") {
                spawn_for::<V>(existing.ucid, global_tx.subscribe(), &insim, &mut active);
            }

            loop {
                tokio::select! {
                    packet = packets.recv() => match packet {
                        Ok(Packet::Ncn(ncn)) => {
                            if active.contains_key(&ncn.ucid) {
                                continue;
                            }

                            spawn_for::<V>(ncn.ucid, global_tx.subscribe(), &insim, &mut active);
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
        Ok::<(), UiError>(())
    });

    let handle = tokio::spawn(async move {
        match thread_handle.join() {
            Ok(result) => result,
            Err(_) => {
                tracing::error!("UI thread panicked");
                Err(UiError::RuntimeCreationFailed)
            },
        }
    });

    (ui_handle, handle)
}

fn spawn_for<V: View>(
    ucid: ConnectionId,
    global_rx: watch::Receiver<V::GlobalProps>,
    insim: &insim::builder::InsimTask,
    active: &mut HashMap<ConnectionId, watch::Sender<V::ConnectionProps>>,
) {
    let (connection_tx, connection_rx) = watch::channel(V::ConnectionProps::default());

    run_view::<V>(ucid, global_rx, connection_rx, insim.clone());
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
