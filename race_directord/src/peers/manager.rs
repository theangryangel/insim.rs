use insim::codec::Frame;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, mpsc, oneshot, Notify};

use super::Peer;
use crate::{config::peer::PeerConfig, InsimConnection, InsimEvent, InsimPacket};

enum ShutdownKind {
    One(String),
    All,
}

enum Command {
    Spawn {
        name: String,
        config: PeerConfig,
    },

    ListPeers {
        respond_to: oneshot::Sender<Vec<String>>,
    },

    Subscribe {
        name: String,
        respond_to: oneshot::Sender<crate::Result<broadcast::Receiver<InsimEvent>>>,
    },

    Shutdown(ShutdownKind),
}

#[derive(Clone)]
pub(crate) struct Manager {
    sender: mpsc::Sender<Command>,
    shutdown_notify: Arc<Notify>,
}

async fn run_manager(mut rx: mpsc::Receiver<Command>, shutdown: Arc<Notify>) -> crate::Result<()> {
    let (shutdown_send, mut shutdown_recv) = mpsc::channel::<String>(256);
    let mut peers: HashMap<String, Peer> = HashMap::new();

    loop {
        tokio::select! {
            res = shutdown_recv.recv() => match res {
                Some(k) => {
                    tracing::debug!("Removing {:?}", &k);
                    peers.remove(&k);
                    if peers.is_empty() {
                        break;
                    }
                },
                None => {
                    break;
                },
            },

            res = rx.recv() => match res {

                Some(Command::Spawn { name, config }) => {

                    let mut max_attempts = usize::MAX;

                    let client = match config {
                        PeerConfig::Relay { auto_select_host, websocket, spectator, admin, connection_attempts } => {
                            if let Some(i) = connection_attempts {
                                max_attempts = i;
                            }

                            InsimConnection::relay(
                                auto_select_host.clone(),
                                websocket,
                                spectator.clone(),
                                admin,
                                InsimPacket::isi_default(),
                            )

                        },
                        PeerConfig::Tcp { addr, connection_attempts } => {
                            if let Some(i) = connection_attempts {
                                max_attempts = i;
                            }

                            InsimConnection::tcp(
                                insim::codec::Mode::Compressed, addr, true, InsimPacket::isi_default(),
                            )
                        },

                        PeerConfig::Udp { bind, addr, connection_attempts } => {
                            if let Some(i) = connection_attempts {
                                max_attempts = i;
                            }

                            InsimConnection::udp(
                                bind, addr, insim::codec::Mode::Compressed, true, InsimPacket::isi_default(),
                            )
                        }
                    };

                    let peer = Peer::new(
                        name.to_string(),
                        client,
                        shutdown_send.clone(),
                        max_attempts,
                    );
                    peers.insert(name.to_string(), peer);
                },

                Some(Command::ListPeers { respond_to }) => {
                    let keys = peers.keys().cloned().collect::<Vec<String>>();
                    let _ = respond_to.send(keys);
                },

                Some(Command::Shutdown(ShutdownKind::All)) => {
                    for (_name, peer) in peers.iter() {
                        peer.shutdown().await?;
                    }
                    break;
                },

                Some(Command::Shutdown(ShutdownKind::One(name))) => {
                    if let Some(peer) = peers.get(&name) {
                        peer.shutdown().await?;
                    }
                },


                Some(Command::Subscribe {
                    name,
                    respond_to,
                }) => {
                    if let Some(peer) = peers.get(&name) {
                        let res = peer.subscribe().await;
                        let _ = respond_to.send(Ok(res));
                    }
                },

                None => {
                    for (_name, peer) in peers.iter() {
                        peer.shutdown().await?;
                    }
                }
            }

        }
    }

    shutdown.notify_waiters();
    Ok(())
}

impl Manager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(16);
        let notify = Arc::new(Notify::new());

        tokio::spawn(run_manager(rx, notify.clone()));

        Self {
            sender: tx,
            shutdown_notify: notify,
        }
    }

    pub async fn shutdown(&mut self) -> crate::Result<()> {
        self.sender
            .send(Command::Shutdown(ShutdownKind::All))
            .await?;
        Ok(())
    }

    pub async fn add_peer(&mut self, name: &str, config: PeerConfig) -> crate::Result<()> {
        self.sender
            .send(Command::Spawn {
                name: name.to_string(),
                config,
            })
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn remove_peer(&mut self, name: &str) -> crate::Result<()> {
        self.sender
            .send(Command::Shutdown(ShutdownKind::One(name.to_string())))
            .await?;
        Ok(())
    }

    pub async fn run(&self) {
        self.shutdown_notify.notified().await;
    }

    pub async fn list(&self) -> Vec<String> {
        let (send, recv) = oneshot::channel();

        let _ = self
            .sender
            .send(Command::ListPeers { respond_to: send })
            .await;

        recv.await.expect("Actor killed")
    }

    pub async fn subscribe(&self, name: &str) -> crate::Result<broadcast::Receiver<InsimEvent>> {
        let (send, recv) = oneshot::channel();

        let _ = self
            .sender
            .send(Command::Subscribe {
                name: name.to_string(),
                respond_to: send,
            })
            .await;

        recv.await.expect("Actor killed")
    }
}
