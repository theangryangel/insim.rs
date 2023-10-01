use std::time::Duration;

use tokio::{
    sync::{broadcast, mpsc, oneshot},
    time::sleep,
};

use crate::{InsimConnection, InsimError, InsimEvent};

pub enum Message {
    Subscribe {
        respond_to: oneshot::Sender<broadcast::Receiver<InsimEvent>>,
    },

    Shutdown,
}

pub(crate) struct Peer {
    sender: mpsc::Sender<Message>,
}

impl Peer {
    pub fn new(
        name: String,
        client: InsimConnection,
        shutdown_tx: mpsc::Sender<String>,
        connection_attempts: usize,
    ) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        tokio::spawn(run_peer(
            name,
            client,
            receiver,
            shutdown_tx,
            connection_attempts,
        ));

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
}

async fn run_peer(
    name: String,
    mut client: InsimConnection,
    mut rx: mpsc::Receiver<Message>,
    shutdown_tx: mpsc::Sender<String>,
    connection_attempts: usize,
) {
    let mut connected = false;
    let mut delay: u64 = 2;
    let mut attempts: usize = 0;
    let (broadcast_tx, _) = broadcast::channel::<InsimEvent>(32);

    loop {
        tokio::select! {

            res = rx.recv() => match res {
                Some(Message::Shutdown) | None => {
                    client.shutdown();
                    tracing::trace!("Shutdown requested");
                    break;
                },
                Some(Message::Subscribe { respond_to }) => {
                    let _ = respond_to.send(broadcast_tx.subscribe());
                },
            },

            res = client.poll() => {
                match res {
                    Ok(InsimEvent::Connected(id)) => {
                        delay = 1;
                        attempts = 0;
                        connected = true;
                        tracing::info!("Connected");
                        let _ = broadcast_tx.send(InsimEvent::Connected(id));
                    },
                    Ok(e) => {
                        tracing::trace!("Event={:?}", e);
                        let _ = broadcast_tx.send(e);
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
                        if !connected {
                            attempts = attempts.wrapping_add(1);

                            if connection_attempts > 0 && attempts >= connection_attempts {
                                tracing::error!("Max connection attempts exceeded");
                                break;
                            }
                            delay = std::cmp::min(delay.wrapping_mul(attempts as u64), 120);
                            tracing::warn!(
                                "Timeout during connection attempt = {}/{}, delaying = {}",
                                attempts,
                                connection_attempts,
                                delay
                            );
                            sleep(Duration::from_secs(delay)).await;
                        }

                        connected = client.is_connected();
                    },
                }
            },

        }
    }

    let _ = shutdown_tx.send(name).await;
}
