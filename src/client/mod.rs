pub(crate) mod config;
pub(crate) mod event_handler;

pub use config::Config;
pub use event_handler::EventHandler;

use super::{error, protocol};

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

#[derive(Debug, Clone)]
pub struct Ctx {
    tx: mpsc::UnboundedSender<protocol::packet::Packet>,
    shutdown: mpsc::UnboundedSender<bool>,
}

// TODO remove this allow unused
#[allow(unused)]
impl Ctx {
    pub fn send(&self, data: protocol::packet::Packet) {
        self.tx.send(data);
    }

    pub fn shutdown(&self) {
        self.shutdown.send(true);
    }
}

pub struct Client {
    config: Arc<config::Config>,

    shutdown: Option<mpsc::UnboundedSender<bool>>,
    tx: Option<mpsc::UnboundedSender<protocol::packet::Packet>>,
}

impl Client {
    pub fn from_config(config: config::Config) -> Self {
        Self {
            config: Arc::new(config),
            tx: None,
            shutdown: None,
        }
    }

    pub async fn run(mut self) -> Result<(), error::Error> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        let (send_tx, mut send_rx) = mpsc::unbounded_channel();

        self.tx = Some(send_tx.clone());
        self.shutdown = Some(shutdown_tx.clone());

        let config = self.config.clone();

        let mut i = 0;

        loop {
            if i >= config.max_reconnect_attempts {
                return Err(error::Error::MaxConnectionAttempts);
            }

            tracing::info!("Connection attempt {:?}", i);

            let inner = self.handshake().await;

            if let Err(e) = inner {
                if !config.reconnect {
                    return Err(e);
                }

                i += 1;
                // TODO add jitter
                // TODO add custom retry pattern
                let delay = tokio::time::sleep(Duration::from_secs((i * 5).into()));

                tokio::select! {
                    Some(_) = shutdown_rx.recv() => { return Ok(()); },
                    _ = delay => { continue }
                }
            }

            let mut inner = inner.unwrap();

            let mut interval = time::interval(Duration::from_secs(15));
            let mut timeout = next_timeout();

            for event_handler in &config.event_handlers {
                event_handler.on_connect(Ctx {
                    tx: send_tx.clone(),
                    shutdown: shutdown_tx.clone(),
                });
            }

            i = 0;

            // TODO turn this inot an inner loop method
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

                        // TODO move this into it's own handler fn of some kind
                        match result {
                            Ok(protocol::packet::Packet::Tiny(protocol::insim::Tiny{ reqi: 0, .. })) => {
                                tracing::debug!("ping? pong!");
                                // keep the connection alive
                                let pong = protocol::packet::Packet::Tiny(protocol::insim::Tiny{
                                    reqi: 0,
                                    subtype: 0,
                                });

                                let res = inner.send(pong).await;
                                if let Err(e) = res {
                                    tracing::error!("failed to send ping response: {:?}", e);
                                    break;
                                }
                            },

                            Ok(protocol::packet::Packet::Version(
                                    protocol::insim::Version{ insimver: version, ..  }
                            )) => {
                                if version != protocol::insim::VERSION {
                                    return Err(error::Error::IncompatibleVersion);
                                }
                            },

                            Ok(frame) => {
                                let ctx = Ctx{tx: send_tx.clone(), shutdown: shutdown_tx.clone()};

                                for event_handler in &config.event_handlers {
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

                            for event_handler in &config.event_handlers {
                                event_handler.on_timeout();
                            }

                            break;
                        }
                    },
                }
            }

            for event_handler in config.event_handlers.iter() {
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
                let isi = protocol::packet::Packet::Init(protocol::insim::Init {
                    name: self.config.name.to_owned().into(),
                    password: self.config.password.to_owned().into(),
                    prefix: self.config.prefix,
                    version: protocol::insim::VERSION,
                    interval: self.config.interval_ms,
                    flags: self.config.flags,
                    reqi: 1,
                });

                let res = inner.send(isi).await;
                if let Err(e) = res {
                    return Err(e.into());
                }
                Ok(inner)
            }
            Err(e) => Err(e),
        }
    }

    // TODO re-add shutdown and send methods at some point, on the off chance we want them on the
    // client directly?
}
