use crate::packets;
use crate::protocol;

use futures::prelude::*;
use std::time::Duration;
use tokio::time;

fn next_timeout() -> time::Instant {
    time::Instant::now() + Duration::from_secs(30)
}

pub struct Client {
    name: String,
    inner: protocol::stream::InsimPacketStream,
    timeout: time::Instant,
}

impl Client {
    // TODO implement some kind of config

    pub async fn new_udp(name: String, dest: String) -> Client {
        let inner = protocol::stream::InsimPacketStream::new_udp(dest).await;

        let mut client = Client {
            name: name.to_owned(),
            inner,
            timeout: next_timeout(),
        };
        client.init().await;
        client
    }

    pub async fn new_tcp(name: String, dest: String) -> Client {
        let inner = protocol::stream::InsimPacketStream::new_tcp(dest).await;

        let mut client = Client {
            name: name.to_owned(),
            inner,
            timeout: next_timeout(),
        };
        client.init().await;
        client
    }

    pub async fn new_relay(name: String) -> Client {
        Client::new_tcp(name, "isrelay.lfs.net:47474".to_string()).await
    }

    async fn init(&mut self) {
        let isi = packets::Insim::Init(packets::insim::Init {
            name: self.name.to_owned().into(),
            password: "".into(),
            prefix: 0,
            version: packets::insim::VERSION,
            interval: 1000,
            flags: (1 << 5), // TODO: implement something better here
            reqi: 1,
        });

        self.send(isi).await;

        // TODO implement unexpected version handling
    }

    pub async fn run(&mut self) {
        let mut interval = time::interval(Duration::from_secs(15));

        loop {
            tokio::select! {
                Some(result) = self.inner.next() => {
                    self.timeout = next_timeout();

                    // TODO move this into it's own handler fn of some kind
                    match result {
                        Ok(packets::Insim::Tiny(packets::insim::Tiny{ reqi: 0, .. })) => {
                            // keep the connection alive
                            println!("ping? pong!");
                            let pong = packets::Insim::Tiny(packets::insim::Tiny{
                                reqi: 0,
                                subtype: 0,
                            });
                            self.send(pong).await;
                        },

                        Ok(frame) => {
                            // TODO remove
                            println!("[recv] {:?}", frame);

                            // TODO event handling of some kind.
                            // Do we throw it out to a channel? or have some highly specific
                            // handler mapping? Or?
                        },

                        // TODO add unknown packet handling to just log an error
                        // after that, switch this to return
                        Err(error) => {
                            println!("[err] {:?}", error);
                        },
                    }
                },

                tick = interval.tick() => {
                    if tick > self.timeout {
                        println!("Timeout!");
                        // TODO add a custom error here
                        return
                    }

                    // TODO remove
                    println!("[tick? tock!] {:?}", tick);
                },

                // TODO add quit/exit handler
            }
        }
    }

    pub async fn send(&mut self, data: packets::Insim) -> std::result::Result<(), std::io::Error> {
        // TODO remove
        println!("[send] {:?}", data);
        self.inner.send(data).await
    }
}
