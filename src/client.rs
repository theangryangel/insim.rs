use crate::packets;
use crate::protocol;

use futures::prelude::*;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

fn next_timeout() -> time::Instant {
    time::Instant::now() + Duration::from_secs(30)
}

// TODO can we reuse the protocol stuff?
// should we reuse the protocol stuff?
pub enum ConnectionType {
    TCP,
    UDP,
}

pub struct Client {
    ctype: ConnectionType,
    name: String,
    dest: String,
}

impl Client {
    // TODO Client is effectively just a config wrapper at this point
    // move run function out of here and rename client to something like ClientConfig

    pub fn new_udp(name: String, dest: String) -> Client {
        Client {
            ctype: ConnectionType::UDP,
            name,
            dest,
        }
    }

    pub fn new_tcp(name: String, dest: String) -> Client {
        Client {
            ctype: ConnectionType::TCP,
            name,
            dest,
        }
    }

    pub fn new_relay(name: String) -> Client {
        Client::new_tcp(name, "isrelay.lfs.net:47474".to_string())
    }

    pub async fn run(
        &self,
    ) -> (
        mpsc::UnboundedSender<bool>,
        mpsc::UnboundedSender<packets::Insim>,
        mpsc::UnboundedReceiver<packets::Insim>,
    ) {
        // TODO add error handling, infinite reconnects, etc.
        // TODO break this up
        // TODO implement unexpected version handling

        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        let (send_tx, mut send_rx) = mpsc::unbounded_channel();
        let (recv_tx, recv_rx) = mpsc::unbounded_channel();

        // TODO move the connection handling into the task runner loop
        let mut inner = match self.ctype {
            ConnectionType::UDP => {
                protocol::stream::InsimPacketStream::new_udp(self.dest.to_owned()).await
            }
            ConnectionType::TCP => {
                protocol::stream::InsimPacketStream::new_tcp(self.dest.to_owned()).await
            }
        };

        let isi = packets::Insim::Init(packets::insim::Init {
            name: self.name.to_owned().into(),
            password: "".into(),
            prefix: 0,
            version: packets::insim::VERSION,
            interval: 1000,
            flags: (1 << 5), // TODO: implement something better here that will eventually pull from ClientConfig
            reqi: 1,
        });

        inner.send(isi).await;

        let mut interval = time::interval(Duration::from_secs(15));
        let mut timeout = next_timeout();

        tokio::spawn(async move {
            loop {
                tokio::select! {

                    Some(_) = shutdown_rx.recv() => {
                        println!("Quitting...");
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

                            Ok(frame) => {
                                recv_tx.send(frame);
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
