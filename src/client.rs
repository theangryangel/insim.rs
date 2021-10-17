use crate::{error, packets, protocol, Config};

use futures::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

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

pub struct Client {
    shutdown: mpsc::UnboundedSender<bool>,
    tx: mpsc::UnboundedSender<packets::Insim>,
    rx: mpsc::UnboundedReceiver<Result<Event, error::Error>>,
}

impl Client {
    pub fn from_config(config: Config) -> Self {
        // TODO add error handling, infinite reconnects, etc.
        // TODO break this up - ClientInner? blehasd.

        let (shutdown_tx, shutdown_rx) = mpsc::unbounded_channel();
        let (send_tx, send_rx) = mpsc::unbounded_channel();
        let (recv_tx, recv_rx) = mpsc::unbounded_channel();

        let conf = Arc::new(config);

        tokio::spawn(worker(conf, send_rx, recv_tx, shutdown_rx));

        Client {
            tx: send_tx,
            rx: recv_rx,
            shutdown: shutdown_tx,
        }
    }

    pub fn send(&self, data: packets::Insim) {
        self.tx.send(data);
    }

    pub async fn recv(&mut self) -> Option<Result<Event, error::Error>> {
        use futures::future::poll_fn;

        poll_fn(|cx| self.rx.poll_recv(cx)).await
    }

    pub fn shutdown(&self) {
        self.shutdown.send(true);
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

async fn worker(
    config: Arc<Config>,

    mut send_rx: mpsc::UnboundedReceiver<packets::Insim>,
    recv_tx: mpsc::UnboundedSender<Result<Event, error::Error>>,
    mut shutdown_rx: mpsc::UnboundedReceiver<bool>,
) {
    let mut i = 0;

    loop {
        if i > config.max_reconnect_attempts {
            recv_tx.send(Err(error::Error::MaxConnectionAttempts));
            return;
        }

        println!("Connecting...");

        let inner = handshake(config.clone()).await;

        if let Err(e) = inner {
            recv_tx.send(Err(e));

            if config.reconnect {
                i += 1;
                continue;
            }

            return;
        }

        let mut inner = inner.unwrap();

        let mut interval = time::interval(Duration::from_secs(15));
        let mut timeout = next_timeout();

        recv_tx.send(Ok(Event::Connected));
        i = 0;

        loop {
            tokio::select! {

                Some(_) = shutdown_rx.recv() => {
                    println!("shutdown");
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
                                packets::insim::Version{ insimver: version, ..  }
                        )) => {
                            if version != packets::insim::VERSION {
                                recv_tx.send(Err(error::Error::IncompatibleVersion));
                                return;
                            }
                        },

                        Ok(frame) => {
                            recv_tx.send(Ok(Event::Raw(frame)));
                        },

                        // TODO add unknown packet handling to just log an error
                        // after that, switch this to return
                        Err(error) => {
                            println!("[err] {:?}", error);
                            panic!("TODO");
                        },
                    }
                },

                tick = interval.tick() => {
                    println!("TICK {:?} TIMEOUT {:?}", tick, timeout);
                    if tick > timeout {
                        recv_tx.send(Err(error::Error::Timeout));
                        println!("TIMEOUT");
                        break;
                    }
                },
            }
        }

        recv_tx.send(Ok(Event::Disconnected));
    }
}
