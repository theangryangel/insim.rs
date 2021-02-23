use futures::prelude::*;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed};
use crate::{codec, proto};

pub struct Client {
    name: String,
    inner: Framed<TcpStream, codec::InsimCodec>,
}

impl Client {
    pub async fn connect(name: String, dest: String) -> Client {
        let stream = TcpStream::connect(dest).await.unwrap();

        let mut inner = Framed::new(
            stream, codec::InsimCodec::new()
        );

        let mut client = Client{
            name: name.to_owned(),
            inner: inner
        };
        client.init().await;
        client
    }

    pub async fn init(&mut self) {
        let isi = proto::Insim::INIT {
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

    pub async fn recv(&mut self) -> Option<std::result::Result<proto::Insim, std::io::Error>>{
        // TODO: This should probably be done with a sink and a stream?
        let result = self.inner.next().await;

        // keep the connection alive
        if let Some(Ok(proto::Insim::TINY{reqi: 0, ..})) = result {
            println!("ping? pong!");
            let pong = proto::Insim::TINY {reqi: 0, subtype:0};
            self.send(pong).await;
        }
        result
    }

    pub async fn send(&mut self, data: proto::Insim) -> std::result::Result<(), std::io::Error> {
       self.inner.send(data).await
    }
}
