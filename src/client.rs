use crate::{codec, proto};
use futures::prelude::*;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct Client {
    name: String,
    inner: Framed<TcpStream, codec::InsimCodec>,
}

impl Client {
    pub async fn connect(name: String, dest: String) -> Client {
        let stream = TcpStream::connect(dest).await.unwrap();

        let inner = Framed::new(stream, codec::InsimCodec::new());

        let mut client = Client {
            name: name.to_owned(),
            inner,
        };
        client.init().await;
        client
    }

    pub async fn init(&mut self) {
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
    }

    pub async fn recv(&mut self) -> Option<std::result::Result<proto::Insim, std::io::Error>> {
        // TODO: This should probably be done with a sink and a stream?
        let result = self.inner.next().await;

        // TODO remove
        println!("[recv] {:?}", result);

        // TODO implement unexpected version handling

        // keep the connection alive
        if let Some(Ok(proto::Insim::Tiny { reqi: 0, .. })) = result {
            println!("ping? pong!");
            let pong = proto::Insim::Tiny {
                reqi: 0,
                subtype: 0,
            };
            self.send(pong).await;
        }

        result
    }

    pub async fn send(&mut self, data: proto::Insim) -> std::result::Result<(), std::io::Error> {
        // TODO remove
        println!("[send] {:?}", data);
        self.inner.send(data).await
    }

    // TODO event handling of some kind.
    // We could do something like the command macros in
    // https://github.com/serenity-rs/serenity?
}
