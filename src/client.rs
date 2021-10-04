use crate::{error, packets, protocol};

use futures::prelude::*;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

fn next_timeout() -> time::Instant {
    time::Instant::now() + Duration::from_secs(30)
}

#[derive(Clone, Debug)]
enum TransportType {
    Tcp,
    Udp,
}

#[derive(Debug)]
pub enum Event {
    Connected,
    Disconnected,

    Raw(packets::Insim),
}

#[derive(Clone, Debug)]
pub struct Client {
    ctype: TransportType,
    name: String,
    host: String,
    password: String,
    flags: u16,
    prefix: u8,
    interval_ms: u16,
}

impl Default for Client {
    fn default() -> Client {
        Client::new()
    }
}

impl Client {
    // Builder functions
    pub fn new() -> Self {
        Self {
            ctype: TransportType::Tcp,
            name: "insim.rs".into(),
            host: "127.0.0.1:29999".into(),
            password: "".into(),
            flags: (1 << 5), // TODO make a builder
            prefix: 0,
            interval_ms: 1000,
        }
    }

    pub fn using_tcp(mut self, host: String) -> Self {
        self.ctype = TransportType::Tcp;
        self.host = host;
        self
    }

    pub fn using_relay(mut self) -> Self {
        self.ctype = TransportType::Tcp;
        self.host = "isrelay.lfs.net:47474".into();
        self
    }

    pub fn using_udp(mut self, host: String) -> Self {
        self.ctype = TransportType::Udp;
        self.host = host;
        self
    }

    pub fn named(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_flags(mut self, flags: u16) -> Self {
        self.flags = flags;
        self
    }

    pub fn with_password(mut self, pwd: String) -> Self {
        self.password = pwd;
        self
    }

    pub fn with_prefix(mut self, prefix: u8) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn with_interval(mut self, interval: u16) -> Self {
        // TODO take a Duration and automatically convert it
        self.interval_ms = interval;
        self
    }

    // Runner
    pub async fn run(
        &self,
    ) -> (
        mpsc::UnboundedSender<bool>,
        mpsc::UnboundedSender<packets::Insim>,
        mpsc::UnboundedReceiver<Result<Event, error::Error>>,
    ) {
        // TODO add error handling, infinite reconnects, etc.
        // TODO break this up
        // TODO implement unexpected version handling

        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        let (send_tx, mut send_rx) = mpsc::unbounded_channel();
        let (recv_tx, recv_rx) = mpsc::unbounded_channel();

        // TODO move the connection handling into the task runner loop
        let mut inner = match self.ctype {
            TransportType::Udp => {
                protocol::stream::InsimPacketStream::new_udp(self.host.to_owned()).await
            }
            TransportType::Tcp => {
                protocol::stream::InsimPacketStream::new_tcp(self.host.to_owned()).await
            }
        };

        let isi = packets::Insim::Init(packets::insim::Init {
            name: self.name.clone().into(),
            password: self.password.clone().into(),
            prefix: self.prefix,
            version: packets::insim::VERSION,
            interval: self.interval_ms,
            flags: self.flags,
            reqi: 1,
        });

        inner.send(isi).await;

        let mut interval = time::interval(Duration::from_secs(15));
        let mut timeout = next_timeout();

        tokio::spawn(async move {
            loop {
                tokio::select! {

                    Some(_) = shutdown_rx.recv() => {
                        recv_tx.send(Ok(Event::Disconnected));
                        return;
                    },

                    Some(packet) = send_rx.recv() => {
                        inner.send(packet).await;
                    },

                    Some(result) = inner.next() => {
                        timeout = next_timeout();

                        // TODO move this into it's own handler fn of some kind
                        match result {
                            Ok(packets::Insim::Tiny(packets::insim::Tiny{ reqi: 0, .. })) => {
                                // keep the connection alive
                                let pong = packets::Insim::Tiny(packets::insim::Tiny{
                                    reqi: 0,
                                    subtype: 0,
                                });

                                inner.send(pong).await;
                            },

                            Ok(packets::Insim::Version(
                                packets::insim::Version{ reqi: 1, insimver: version, ..  }
                            )) => {
                                if version != packets::insim::VERSION {
                                    // TODO return a custom Err rather than panic
                                    panic!("Unsupported Insim Version! Found {:?} expected {:?}", version, packets::insim::VERSION);
                                }

                                recv_tx.send(Ok(Event::Connected));
                            },

                            Ok(frame) => {
                                //recv_tx.send(frame);

                                recv_tx.send(Ok(Event::Raw(frame)));
                            },

                            // TODO add unknown packet handling to just log an error
                            // after that, switch this to return
                            Err(error) => {
                                println!("[err] {:?}", error);
                            },
                        }
                    },

                    tick = interval.tick() => {
                        if tick > timeout {
                            println!("Timeout!");
                            // TODO add a custom error here
                            return
                        }
                    },
                }
            }
        });

        (shutdown_tx, send_tx, recv_rx)
    }
}
