use bytes::BytesMut;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;

use super::AsyncTryReadWriteBytes;
use crate::{error::Error, result::Result};

pub(crate) type TungsteniteWebSocket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

pub(crate) async fn connect_to_relay(tcp_nodelay: bool) -> Result<TungsteniteWebSocket> {
    use tokio_tungstenite::{
        connect_async_with_config,
        tungstenite::{handshake::client::generate_key, http},
    };

    let uri = format!("ws://{}/connect", crate::LFSW_RELAY_ADDR)
        .parse::<http::Uri>()
        .expect("Failed to parse relay URI");

    let req = http::Request::builder()
        .method("GET")
        .header("Host", uri.host().expect("Failed to get host from uri"))
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", generate_key())
        // It appears that isrelay.lfs.net requires an Origin header
        // Without this it does not allow us to connect.
        .header("Origin", "null")
        .uri(uri)
        .body(())
        .unwrap();

    let (stream, _response) = connect_async_with_config(req, None, tcp_nodelay).await?;

    Ok(stream)
}

#[async_trait::async_trait]
impl AsyncTryReadWriteBytes for TungsteniteWebSocket {
    async fn try_read_bytes(&mut self, buf: &mut BytesMut) -> Result<usize> {
        use tokio_tungstenite::tungstenite::Message;

        // loop because we might get non-binary packets
        // which we need to ignore
        loop {
            match self.next().await {
                Some(Ok(Message::Binary(data))) => {
                    buf.extend_from_slice(&data);
                    return Ok(data.len());
                },
                Some(Ok(Message::Close(_))) => {
                    return Err(Error::Disconnected);
                },
                Some(Ok(msg)) => {
                    tracing::debug!(
                        "Ignoring non-binary packet received over websocket: {:?}",
                        msg
                    );
                },
                Some(Err(e)) => return Err(e.into()),
                None => return Err(Error::Disconnected),
            }
        }
    }

    async fn try_write_bytes(&mut self, src: &[u8]) -> Result<usize> {
        let msg = tokio_tungstenite::tungstenite::protocol::Message::binary(src);
        self.send(msg).await.unwrap();
        Ok(src.len())
    }
}
