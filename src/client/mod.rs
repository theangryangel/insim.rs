pub(crate) mod config;
pub(crate) mod event_handler;

pub use config::Config;
pub use event_handler::EventHandler;

use super::{error, protocol};

use futures::prelude::*;
use rand::{self, Rng};
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

#[derive(Debug, Clone)]
pub struct Ctx {
    tx: mpsc::UnboundedSender<protocol::Packet>,
    shutdown: mpsc::UnboundedSender<bool>,
}

#[allow(unused)]
impl Ctx {
    pub fn send(&self, data: protocol::Packet) {
        self.tx.send(data);
    }

    pub fn shutdown(&self) {
        self.shutdown.send(true);
    }
}

pub struct Client {
    config: config::Config,

    shutdown: Option<mpsc::UnboundedSender<bool>>,
    tx: Option<mpsc::UnboundedSender<protocol::Packet>>,
}

impl Client {
    pub fn from_config(config: config::Config) -> Self {
        Self {
            config,
            tx: None,
            shutdown: None,
        }
    }

    pub async fn run(mut self) -> Result<(), error::Error> {
        let (shutdown_tx, shutdown_rx) = mpsc::unbounded_channel();
        let (send_tx, send_rx) = mpsc::unbounded_channel();

        self.tx = Some(send_tx.clone());
        self.shutdown = Some(shutdown_tx.clone());

        for event_handler in self.config.event_handlers.iter() {
            event_handler.on_startup();
        }

        let res = self
            .inner_loop(shutdown_rx, shutdown_tx, send_rx, send_tx)
            .await;

        for event_handler in self.config.event_handlers.iter() {
            event_handler.on_shutdown();
        }

        res
    }

    async fn inner_loop(
        &self,
        mut shutdown_rx: mpsc::UnboundedReceiver<bool>,
        shutdown_tx: mpsc::UnboundedSender<bool>,
        mut send_rx: mpsc::UnboundedReceiver<protocol::Packet>,
        send_tx: mpsc::UnboundedSender<protocol::Packet>,
    ) -> Result<(), error::Error> {
        let mut connection_attempt = 0;

        loop {
            if connection_attempt >= self.config.max_reconnect_attempts {
                return Err(error::Error::MaxConnectionAttempts);
            }

            tracing::debug!("connection attempt {:?}", connection_attempt);

            let inner = self.handshake().await;

            if let Err(e) = inner {
                if !self.config.reconnect {
                    return Err(e);
                }

                connection_attempt += 1;

                let retry_in = self.delay_with_jitter(connection_attempt);
                tracing::debug!("attempting reconnect in {:?}", retry_in);

                tokio::select! {
                    Some(_) = shutdown_rx.recv() => { return Ok(()); },
                    _ = tokio::time::sleep(retry_in) => { continue }
                }
            }

            let mut inner = inner.unwrap();

            let mut interval = time::interval(Duration::from_secs(15));
            let mut timeout = next_timeout();

            for event_handler in self.config.event_handlers.iter() {
                event_handler.on_connect(Ctx {
                    tx: send_tx.clone(),
                    shutdown: shutdown_tx.clone(),
                });
            }

            connection_attempt = 0;

            // TODO turn this into an inner-inner loop method at some point
            loop {
                tokio::select! {

                    Some(_) = shutdown_rx.recv() => {
                        tracing::debug!("shutdown requested");
                        return Ok(());
                    },

                    Some(packet) = send_rx.recv() => {
                        let res = inner.send(packet).await;
                        if let Err(e) = res {
                            return Err(e.into());
                        }
                    },

                    Some(result) = inner.next() => {
                        timeout = next_timeout();

                        match result {
                            Ok(protocol::Packet::Tiny(protocol::insim::Tiny{ reqi: 0, .. })) => {
                                // keep the connection alive
                                tracing::debug!("ping? pong!");

                                let res = inner.send(
                                    protocol::Packet::from(
                                        protocol::insim::Tiny{
                                            reqi: 0,
                                            subtype: 0,
                                        }
                                    )
                                ).await;
                                if let Err(e) = res {
                                    tracing::error!("failed to send ping response: {:?}", e);
                                    break;
                                }
                            },

                            Ok(protocol::Packet::Version(
                                protocol::insim::Version{ insimver: version, ..  }
                            )) => {
                                if version != protocol::insim::VERSION {
                                    return Err(error::Error::IncompatibleVersion);
                                }
                            },

                            Ok(frame) => {
                                let ctx = Ctx{tx: send_tx.clone(), shutdown: shutdown_tx.clone()};

                                for event_handler in self.config.event_handlers.iter() {
                                    event_handler.on_raw(ctx.clone(), &frame);
                                }
                            },

                            Err(e) => {
                                tracing::error!("unhandled error: {:?}", e);
                                break;
                            },
                        }
                    },

                    tick = interval.tick() => {
                        if tick > timeout {
                            tracing::error!("timeout occurred expected by tick {:?}, reached {:?}", tick, timeout);

                            for event_handler in self.config.event_handlers.iter() {
                                event_handler.on_timeout();
                            }

                            break;
                        }
                    },
                }
            }

            for event_handler in self.config.event_handlers.iter() {
                event_handler.on_disconnect();
            }
        }
    }

    async fn handshake(&self) -> Result<protocol::stream::Socket, error::Error> {
        let res = match self.config.ctype {
            TransportType::Udp => {
                protocol::stream::Socket::new_udp(self.config.host.to_owned()).await
            }
            TransportType::Tcp => {
                protocol::stream::Socket::new_tcp(self.config.host.to_owned()).await
            }
        };

        match res {
            Ok(mut inner) => {
                let isi = protocol::insim::Init {
                    name: self.config.name.to_owned().into(),
                    password: self.config.password.to_owned().into(),
                    prefix: self.config.prefix,
                    version: protocol::insim::VERSION,
                    interval: self.config.interval_ms,
                    flags: self.config.flags,
                    reqi: 1,
                };

                let res = inner.send(protocol::Packet::from(isi)).await;
                if let Err(e) = res {
                    return Err(e.into());
                }
                Ok(inner)
            }
            Err(e) => Err(e),
        }
    }

    fn delay_with_jitter(&self, attempt: u16) -> Duration {
        let mut rng = rand::thread_rng();
        Duration::from_millis((rng.gen_range(0..=1000) + (attempt * 5000)).into())
    }

    // TODO re-add shutdown and send methods at some point, on the off chance we want them on the
    // client directly?
}
