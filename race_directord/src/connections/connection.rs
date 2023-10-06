use std::time::Duration;

use tokio::{
    sync::{broadcast, mpsc, oneshot},
    time::sleep,
};

use crate::{InsimConnection, InsimError, InsimEvent};

#[derive(Debug)]
pub enum Message {
    Subscribe {
        respond_to: oneshot::Sender<broadcast::Receiver<InsimEvent>>,
    },

    Shutdown,
}

pub(crate) struct Connection {
    sender: mpsc::Sender<Message>,
}

impl Connection {
    pub fn new(client: InsimConnection, connection_attempts: usize) -> Self {
        let (sender, receiver) = mpsc::channel(8);

        let actor = ConnectionActor::new(client, receiver, connection_attempts);

        tokio::spawn(actor.run());

        Self { sender }
    }

    pub async fn shutdown(&self) -> crate::Result<()> {
        self.sender.send(Message::Shutdown).await?;
        Ok(())
    }

    pub async fn subscribe(&self) -> broadcast::Receiver<InsimEvent> {
        let (send, recv) = oneshot::channel();
        let _ = self
            .sender
            .send(Message::Subscribe { respond_to: send })
            .await;

        recv.await.expect("Actor killed")
    }

    pub(crate) fn is_alive(&self) -> bool {
        !self.sender.is_closed()
    }
}

struct ConnectionActor {
    client: InsimConnection,
    rx: mpsc::Receiver<Message>,
    connection_attempts: usize,
    connected: bool,
    delay: u64,
    attempts: usize,
    bcast: broadcast::Sender<InsimEvent>,
}

impl ConnectionActor {
    fn new(
        client: InsimConnection,
        rx: mpsc::Receiver<Message>,
        connection_attempts: usize,
    ) -> Self {
        let (bcast, _) = broadcast::channel::<InsimEvent>(32);

        Self {
            client,
            rx,
            connection_attempts,
            connected: false,
            delay: 2,
            attempts: 0,
            bcast,
        }
    }

    async fn run(mut self) {
        loop {
            tokio::select! {

                res = self.rx.recv() => match res {
                    Some(Message::Shutdown) | None => {
                        self.client.shutdown();
                        tracing::trace!("Shutdown requested");
                        break;
                    },
                    Some(Message::Subscribe { respond_to }) => {
                        let _ = respond_to.send(self.bcast.subscribe());
                    },
                },

                res = self.client.poll() => {
                    match res {
                        Ok(InsimEvent::Connected(id)) => {
                            self.delay = 1;
                            self.attempts = 0;
                            self.connected = true;
                            tracing::info!("Connected");
                            let _ = self.bcast.send(InsimEvent::Connected(id));
                        },
                        Ok(e) => {
                            tracing::trace!("Event={:?}", e);
                            let _ = self.bcast.send(e);
                        },
                        Err(InsimError::Shutdown) => {
                            tracing::info!("Shutdown");
                            break;
                        },
                        Err(InsimError::IncompatibleVersion(_)) => {
                            tracing::error!("{}", res.unwrap_err());
                            break;
                        },
                        Err(InsimError::Timeout(_)) | Err(InsimError::IO { .. }) | Err(_) => {
                            tracing::error!("{:?}", res.unwrap_err());
                            if !self.connected {
                                self.attempts = self.attempts.wrapping_add(1);

                                if self.connection_attempts > 0 && self.attempts >= self.connection_attempts {
                                    tracing::error!("Max connection attempts exceeded");
                                    break;
                                }
                                self.delay = std::cmp::min(self.delay.wrapping_mul(self.attempts as u64), 120);
                                tracing::warn!(
                                    "Timeout during connection attempt = {}/{}, delaying = {}",
                                    self.attempts,
                                    self.connection_attempts,
                                    self.delay
                                );
                                sleep(Duration::from_secs(self.delay)).await;
                            }

                            self.connected = self.client.is_connected();
                        },
                    }
                },

            }
        }
    }
}
