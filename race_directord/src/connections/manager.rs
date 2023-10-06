use insim::codec::Frame;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::{broadcast, mpsc, oneshot, Notify},
    time::interval,
};

use super::Connection;
use crate::{config::connection::ConnectionConfig, InsimConnection, InsimEvent, InsimPacket};

#[derive(Debug)]
enum ConnectionManagerControl {
    Spawn {
        name: String,
        config: ConnectionConfig,
    },

    Kill(String),

    List {
        respond_to: oneshot::Sender<Vec<String>>,
    },

    Subscribe {
        name: String,
        respond_to: oneshot::Sender<crate::Result<broadcast::Receiver<InsimEvent>>>,
    },

    Shutdown,
}

#[derive(Clone)]
pub(crate) struct ConnectionManager {
    sender: mpsc::Sender<ConnectionManagerControl>,
    shutdown_notify: Arc<Notify>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(16);
        let notify = Arc::new(Notify::new());

        let actor = ConnectionManagerActor::new(rx, notify.clone());
        tokio::spawn(actor.run());

        Self {
            sender: tx,
            shutdown_notify: notify,
        }
    }

    pub async fn shutdown(&mut self) -> crate::Result<()> {
        self.sender.send(ConnectionManagerControl::Shutdown).await?;
        Ok(())
    }

    pub async fn add_peer(&mut self, name: &str, config: ConnectionConfig) -> crate::Result<()> {
        self.sender
            .send(ConnectionManagerControl::Spawn {
                name: name.to_string(),
                config,
            })
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn remove_peer(&mut self, name: &str) -> crate::Result<()> {
        self.sender
            .send(ConnectionManagerControl::Kill(name.to_string()))
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
            .send(ConnectionManagerControl::List { respond_to: send })
            .await;

        recv.await.expect("Actor killed")
    }

    pub async fn subscribe(&self, name: &str) -> crate::Result<broadcast::Receiver<InsimEvent>> {
        let (send, recv) = oneshot::channel();

        let _ = self
            .sender
            .send(ConnectionManagerControl::Subscribe {
                name: name.to_string(),
                respond_to: send,
            })
            .await;

        recv.await.expect("Actor killed")
    }
}

struct ConnectionManagerActor {
    rx: mpsc::Receiver<ConnectionManagerControl>,
    shutdown: Arc<Notify>,

    peers: HashMap<String, Connection>,
}

impl ConnectionManagerActor {
    fn new(rx: mpsc::Receiver<ConnectionManagerControl>, shutdown: Arc<Notify>) -> Self {
        ConnectionManagerActor {
            rx,
            shutdown,
            peers: HashMap::default(),
        }
    }

    fn handle_heartbeat(&mut self) {
        self.peers.retain(|_, v| v.is_alive());
    }

    fn handle_peer_spawn(&mut self, name: String, config: ConnectionConfig) {
        let mut max_attempts = usize::MAX;

        let client = match config {
            ConnectionConfig::Relay {
                auto_select_host,
                websocket,
                spectator,
                admin,
                connection_attempts,
            } => {
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
            }
            ConnectionConfig::Tcp {
                addr,
                connection_attempts,
            } => {
                if let Some(i) = connection_attempts {
                    max_attempts = i;
                }

                InsimConnection::tcp(
                    insim::codec::Mode::Compressed,
                    addr,
                    true,
                    InsimPacket::isi_default(),
                )
            }

            ConnectionConfig::Udp {
                bind,
                addr,
                connection_attempts,
            } => {
                if let Some(i) = connection_attempts {
                    max_attempts = i;
                }

                InsimConnection::udp(
                    bind,
                    addr,
                    insim::codec::Mode::Compressed,
                    true,
                    InsimPacket::isi_default(),
                )
            }
        };

        let peer = Connection::new(client, max_attempts);
        self.peers.insert(name.to_string(), peer);
    }

    async fn run(mut self) -> crate::Result<()> {
        let mut interval = interval(Duration::from_secs(90));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.handle_heartbeat();
                },

                res = self.rx.recv() => match res {

                    Some(ConnectionManagerControl::Spawn { name, config }) => {
                        self.handle_peer_spawn(name, config);
                    },

                    Some(ConnectionManagerControl::List { respond_to }) => {
                        let keys = self.peers.keys().cloned().collect::<Vec<String>>();
                        let _ = respond_to.send(keys);
                    },

                    Some(ConnectionManagerControl::Kill(name)) => {
                        if let Some(peer) = self.peers.get(&name) {
                            peer.shutdown().await?;
                            self.peers.remove(&name);
                        }
                    },

                    Some(ConnectionManagerControl::Subscribe {
                        name,
                        respond_to,
                    }) => {
                        if let Some(peer) = self.peers.get(&name) {
                            let res = peer.subscribe().await;
                            let _ = respond_to.send(Ok(res));
                        }
                    },

                    None | Some(ConnectionManagerControl::Shutdown) => {
                        for (_name, peer) in self.peers.iter() {
                            peer.shutdown().await?;
                        }

                        break;
                    }
                }
            }
        }

        self.shutdown.notify_waiters();
        Ok(())
    }
}
