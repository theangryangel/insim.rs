use bytes::{Buf, BytesMut};
use insim_core::Decodable;
use tokio::net::TcpStream;

use crate::codec::{Codec, Mode};
use crate::error::Error;
use crate::packets::Packet;
use crate::result::Result;

use crate::traits::{ReadPacket, ReadWritePacket, WritePacket};
use futures_util::{SinkExt, StreamExt};

pub type TungsteniteWebSocket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

pub struct WebSocket {
    inner: TungsteniteWebSocket,
    codec: Codec,
}

impl WebSocket {
    pub fn new(socket: TungsteniteWebSocket, mode: Mode) -> Self {
        let codec = Codec { mode };

        Self {
            inner: socket,
            codec,
        }
    }
}

impl ReadWritePacket for WebSocket {}

#[async_trait::async_trait]
impl ReadPacket for WebSocket {
    async fn read(&mut self) -> Result<Packet> {
        use tokio_tungstenite::tungstenite::Message;

        // loop because we might get non-binary packets
        // which we need to ignore
        loop {
            match self.inner.next().await {
                Some(Ok(Message::Binary(data))) => {
                    let mut buffer = BytesMut::from(&data[..]);
                    // Websocket packets from Insim are never fragmented
                    // This is handled by Tungstenite.
                    // So we can just skip over using the codec to decode.
                    //
                    // TODO: We should probably actually verify the length.
                    buffer.advance(1);
                    return Ok(Packet::decode(&mut buffer, None)?);
                }
                Some(Ok(Message::Close(_))) => {
                    return Err(Error::Disconnected);
                }
                Some(Ok(msg)) => {
                    tracing::debug!(
                        "Ignoring non-binary packet received over websocket: {:?}",
                        msg
                    );
                }
                Some(Err(e)) => return Err(e.into()),
                None => return Err(Error::Disconnected),
            }
        }
    }
}

#[async_trait::async_trait]
impl WritePacket for WebSocket {
    async fn write(&mut self, packet: Packet) -> Result<()> {
        let mut buf = BytesMut::new();

        self.codec.encode(packet, &mut buf)?;

        let msg = tokio_tungstenite::tungstenite::protocol::Message::binary(&buf[..]);
        self.inner.send(msg).await.unwrap();

        Ok(())
    }
}
