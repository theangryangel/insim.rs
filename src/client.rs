use crate::{error, packets, protocol, Config};

use futures::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use tracing;

fn next_timeout() -> time::Instant {
    time::Instant::now() + Duration::from_secs(30)
}

#[derive(Clone, Debug)]
pub enum TransportType {
    Tcp,
    Udp,
}

#[derive(Debug)]
pub enum Event {
    Connected,
    Disconnected,

    Raw(packets::Insim),
}

#[derive(Debug)]
pub struct Ctx {
    tx: mpsc::UnboundedSender<packets::Insim>,
    shutdown: mpsc::UnboundedSender<bool>,
}

impl Ctx {
    pub fn send(&self, data: packets::Insim) {
        self.tx.send(data);
    }

    pub fn shutdown(&self) {
        self.shutdown.send(true);
    }
}

pub struct Client {
    config: Arc<Config>,

    shutdown: Option<mpsc::UnboundedSender<bool>>,
    tx: Option<mpsc::UnboundedSender<packets::Insim>>,
}

impl Client {
    pub fn from_config(config: Config) -> Self {
        let client = Client {
            config: Arc::new(config),
            tx: None,
            shutdown: None,
        };

        client
    }

    pub async fn run(mut self) -> Result<(), error::Error> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        let (send_tx, mut send_rx) = mpsc::unbounded_channel();

        self.tx = Some(send_tx.clone());
        self.shutdown = Some(shutdown_tx.clone());

        let config = self.config.clone();

        let mut i = 0;

        loop {
            if i > config.max_reconnect_attempts {
                return Err(error::Error::MaxConnectionAttempts);
            }

            tracing::info!("Connection attempt {:?}", i);

            let inner = handshake(config.clone()).await;

            if let Err(e) = inner {
                if !config.reconnect {
                    return Err(e);
                }

                i += 1;
                let delay = tokio::time::sleep(Duration::from_secs((i * 5).into()));

                tokio::select! {
                    Some(_) = shutdown_rx.recv() => { return Ok(()); },
                    _ = delay => { continue }
                }
            }

            let mut inner = inner.unwrap();

            let mut interval = time::interval(Duration::from_secs(15));
            let mut timeout = next_timeout();

            if let Some(event_handler) = &config.event_handler {
                let event_handler = Arc::clone(event_handler);
                event_handler.connected(Ctx {
                    tx: send_tx.clone(),
                    shutdown: shutdown_tx.clone(),
                });
            }

            i = 0;

            loop {
                tokio::select! {

                    Some(_) = shutdown_rx.recv() => {
                        tracing::debug!("shutdown requested");
                        return Ok(());
                    },

                    Some(packet) = send_rx.recv() => {
                        inner.send(packet).await;
                    },

                    Some(result) = inner.next() => {
                        timeout = next_timeout();

                        // TODO move this into it's own handler fn of some kind
                        match result {
                            Ok(packets::Insim::Tiny(packets::insim::Tiny{ reqi: 0, .. })) => {
                                tracing::info!("Ping? Pong!");
                                // keep the connection alive
                                let pong = packets::Insim::Tiny(packets::insim::Tiny{
                                    reqi: 0,
                                    subtype: 0,
                                });

                                inner.send(pong).await;
                            },

                            Ok(packets::Insim::Version(
                                    packets::insim::Version{ insimver: version, ..  }
                            )) => {
                                if version != packets::insim::VERSION {
                                    return Err(error::Error::IncompatibleVersion);
                                }
                            },

                            Ok(frame) => {
                                if let Some(event_handler) = &config.event_handler {
                                    let event_handler = Arc::clone(event_handler);
                                    event_handler.raw(Ctx{tx: send_tx.clone(), shutdown: shutdown_tx.clone()}, frame);
                                }
                            },

                            Err(error) => {
                                return Err(error.into());
                            },
                        }
                    },

                    tick = interval.tick() => {
                        if tick > timeout {
                            tracing::error!("Timeout occurred tick={:?}, timeout={:?}", tick, timeout);

                            if let Some(event_handler) = &config.event_handler {
                                let event_handler = Arc::clone(event_handler);
                                event_handler.timeout();
                            }

                            break;
                        }
                    },
                }
            }

            if let Some(event_handler) = &config.event_handler {
                let event_handler = Arc::clone(event_handler);
                event_handler.disconnected();
            }
        }
    }
}

async fn handshake(config: Arc<Config>) -> Result<protocol::stream::Socket, error::Error> {
    // connect
    let res = match config.ctype {
        TransportType::Udp => protocol::stream::Socket::new_udp(config.host.to_owned()).await,
        TransportType::Tcp => protocol::stream::Socket::new_tcp(config.host.to_owned()).await,
    };

    match res {
        Ok(mut inner) => {
            let isi = packets::Insim::Init(packets::insim::Init {
                name: config.name.to_owned().into(),
                password: config.password.to_owned().into(),
                prefix: config.prefix,
                version: packets::insim::VERSION,
                interval: config.interval_ms,
                flags: config.flags,
                reqi: 1,
            });

            inner.send(isi).await;
            Ok(inner)
        }
        Err(e) => Err(e),
    }
}
