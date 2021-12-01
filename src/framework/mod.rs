pub(crate) mod config;
pub(crate) mod macros;

pub use config::Config;

use super::{error, protocol};

use futures::prelude::*;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing;

pub struct Client {
    config: Arc<config::Config>,

    shutdown: Option<mpsc::UnboundedSender<bool>>,
    tx: Option<mpsc::UnboundedSender<protocol::Packet>>,
}

impl Client {
    pub fn from_config(config: config::Config) -> Self {
        Self {
            config: Arc::new(config),
            tx: None,
            shutdown: None,
        }
    }

    #[allow(unused_must_use)] // if this fails then the we're probably going to die anyway
    pub fn send(&self, data: protocol::Packet) {
        if let Some(tx) = &self.tx {
            tx.send(data);
        }
    }

    #[allow(unused_must_use)] // if this fails then the we're probably going to die anyway
    pub fn shutdown(&self) {
        if let Some(shutdown) = &self.shutdown {
            shutdown.send(true);
        }
    }

    pub async fn run(mut self) -> Result<(), error::Error> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        let (send_tx, mut send_rx) = mpsc::unbounded_channel();

        self.tx = Some(send_tx);
        self.shutdown = Some(shutdown_tx);

        for event_handler in self.config.event_handlers.iter() {
            event_handler.on_startup();
        }

        let hname = &self.config.host;
        let tcp: TcpStream = TcpStream::connect(hname).await.unwrap();

        // TODO handle connection error

        let mut transport = protocol::transport::Transport::new(tcp, self.config.codec_mode);
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

        for event_handler in self.config.event_handlers.iter() {
            event_handler.on_connect(&self);
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
                            for event_handler in self.config.event_handlers.iter() {
                                event_handler.on_raw(&self, &frame);
                            }

                            self.on_packet(&frame);
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

        self.tx = None;
        self.shutdown = None;

        ret
    }
}

use crate::event_handler;
use crate::protocol::Packet;

event_handler!(
    #[allow(unused)]
    pub trait EventHandler for Client, Packet {
        Tiny(protocol::insim::Tiny) => on_tiny,
        MessageOut(protocol::insim::MessageOut) => on_message,
        Npl(protocol::insim::Npl) => on_npl,
        MultiCarInfo(protocol::insim::MultiCarInfo) => on_mci,
        Contact(protocol::insim::Contact) => on_contact,
    }
);
