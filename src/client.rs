use crate::{codec, proto};
use futures::prelude::*;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use tokio::time;
use std::time::Duration;

pub struct Client {
    name: String,
    inner: Framed<TcpStream, codec::InsimCodec>,
    timeout: time::Instant,
}

fn next_timeout() -> time::Instant {
    time::Instant::now() + Duration::from_secs(30)
}

impl Client {

    pub async fn new(name: String, dest: String) -> Client {
        let stream = TcpStream::connect(dest).await.unwrap();

        let inner = Framed::new(stream, codec::InsimCodec::new());

        let mut client = Client {
            name: name.to_owned(),
            inner,
            timeout: next_timeout(),
        };
        client.init().await;
        client
    }

    async fn init(&mut self) {
        let isi = proto::Insim::Init {
            name: self.name.to_owned(),
            password: "".to_string(),
            prefix: b'!',
            version: 8,
            interval: 0,
            flags: 0,
            udpport: 0,
            reqi: 0,
            zero: 0,
        };

        self.send(isi).await;

        // TODO implement unexpected version handling
    }

    pub async fn run(&mut self) {
        let mut interval = time::interval(Duration::from_secs(15));

        loop {
            tokio::select! {
                Some(result) = self.inner.next() => {

                    // TODO move this into it's own handler fn of some kind
                    match result {
                        Ok(proto::Insim::Tiny { reqi: 0, .. }) => {
                            // keep the connection alive
                            println!("ping? pong!");
                            let pong = proto::Insim::Tiny {
                                reqi: 0,
                                subtype: 0,
                            };
                            self.send(pong).await;
                        },

                        Ok(frame) => {
                            self.timeout = next_timeout();

                            // TODO remove
                            println!("[recv] {:?}", frame);

                            // TODO event handling of some kind.
                            // We could do something like the command macros in
                            // https://github.com/serenity-rs/serenity?
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
                        return
                    }

                    // TODO remove
                    println!("TICK {:?}", tick);
                }
            }
        }
    }

    pub async fn send(&mut self, data: proto::Insim) -> std::result::Result<(), std::io::Error> {
        // TODO remove
        println!("[send] {:?}", data);
        self.inner.send(data).await
    }
}
