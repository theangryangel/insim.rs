pub(crate) mod config;
pub(crate) mod event_handler;

pub use config::Config;
pub use event_handler::EventHandler;

use super::{error, protocol};

use futures::prelude::*;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing;

// TODO: Remove this and replace it with something more idomatic and/or magic macro generating
// stuff
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
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        let (send_tx, mut send_rx) = mpsc::unbounded_channel();

        self.tx = Some(send_tx.clone());
        self.shutdown = Some(shutdown_tx.clone());

        for event_handler in self.config.event_handlers.iter() {
            event_handler.on_startup();
        }

        let hname = self.config.host;
        let tcp: TcpStream = TcpStream::connect(hname).await.unwrap();

        // TODO handle connection error

        let mut transport = protocol::transport::Transport::new(tcp);
        let isi = protocol::insim::Init {
            name: self.config.name.to_owned().into(),
            password: self.config.password.to_owned().into(),
            prefix: self.config.prefix,
            version: protocol::insim::VERSION,
            interval: self.config.interval_ms,
            flags: self.config.flags,
            reqi: 1,
        };

        let res = transport.send(isi).await;
        if let Err(e) = res {
            return Err(e.into());
        }

        // TODO handle handshake errors

        let ctx = Ctx {
            tx: send_tx.clone(),
            shutdown: shutdown_tx.clone(),
        };

        for event_handler in self.config.event_handlers.iter() {
            event_handler.on_connect(ctx.clone());
        }

        let mut ret: Result<(), error::Error> = Ok(());

        loop {
            tokio::select! {
                Some(_) = shutdown_rx.recv() => {
                    tracing::debug!("shutdown requested");
                    break;
                },

                Some(frame) = send_rx.recv() => {
                    if let Err(e) = transport.send(frame).await {
                        ret = Err(e.into());
                        break;
                    }
                }

                Some(result) = transport.next() => {

                    match result {
                        Ok(frame) => {
                            let ctx = Ctx{tx: send_tx.clone(), shutdown: shutdown_tx.clone()};

                            for event_handler in self.config.event_handlers.iter() {
                                event_handler.on_raw(ctx.clone(), &frame);
                            }
                        },
                        Err(e) => {
                            ret = Err(e);
                            break;
                        }
                    }

                }
            }
        }

        for event_handler in self.config.event_handlers.iter() {
            event_handler.on_shutdown();
        }

        ret
    }
}
