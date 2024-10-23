use core::{
    pin::Pin,
    task::{Context, Poll},
};
use std::io;

use bytes::{Buf, BytesMut};
use futures_util::{
    sink::{Sink, SinkExt},
    stream::Stream,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::TcpStream,
};
use tokio_tungstenite::tungstenite::{error::Error as TungsteniteError, protocol::Message};

use crate::MAX_SIZE_PACKET;

pub(crate) type TungsteniteWebSocket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

/// Connect to the LFS World Relay over websocket
pub async fn connect_to_lfsworld_relay_ws(
    tcp_nodelay: bool,
) -> std::result::Result<TungsteniteWebSocket, std::io::Error> {
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

    let (stream, _response) = connect_async_with_config(req, None, tcp_nodelay)
        .await
        .map_err(tungstenite_error_to_io)?;

    Ok(stream)
}

/// Convert from Tungstenite error to io::Error
fn tungstenite_error_to_io(err: TungsteniteError) -> io::Error {
    match err {
        TungsteniteError::Io(io_err) => io_err,
        err => io::Error::new(io::ErrorKind::Other, err),
    }
}

impl From<TungsteniteWebSocket> for WebsocketStream {
    fn from(value: TungsteniteWebSocket) -> Self {
        Self {
            inner: value,
            buf: BytesMut::with_capacity(MAX_SIZE_PACKET),
        }
    }
}

/// Binary only Websockets used for LFS World Relay that implements [AsyncRead] and [AsyncWrite]
#[derive(Debug)]
pub struct WebsocketStream {
    inner: TungsteniteWebSocket,
    buf: BytesMut,
}

impl AsyncWrite for WebsocketStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        // TODO: Should we be wrapping stream and then polling that?
        match self.inner.poll_ready_unpin(cx) {
            Poll::Ready(Ok(())) => {
                if let Err(e) = self.inner.start_send_unpin(Message::binary(buf)) {
                    Poll::Ready(Err(tungstenite_error_to_io(e)))
                } else {
                    let _ = self.poll_flush(cx);
                    Poll::Ready(Ok(buf.len()))
                }
            },
            Poll::Ready(Err(e)) => Poll::Ready(Err(tungstenite_error_to_io(e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.inner
            .poll_flush_unpin(cx)
            .map_err(tungstenite_error_to_io)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        let ws = Pin::new(&mut self.inner);
        ws.poll_close(cx).map_err(tungstenite_error_to_io)
    }
}

impl AsyncRead for WebsocketStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            if !self.buf.is_empty() && buf.remaining() > 0 {
                let to_copy = buf.remaining().min(self.buf.len());

                let bytes = self.buf.copy_to_bytes(to_copy);
                buf.put_slice(&bytes);
                return Poll::Ready(Ok(()));
            }

            let ws = Pin::new(&mut self.inner);
            match ws.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Ready(Some(Err(e))) => match e {
                    TungsteniteError::Io(e) => {
                        return Poll::Ready(Err(e));
                    },
                    _ => {
                        return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e)));
                    },
                },
                Poll::Ready(Some(Ok(Message::Binary(msgbuf)))) => {
                    self.buf.extend(msgbuf);
                    continue;
                },
                Poll::Ready(Some(Ok(data))) => {
                    tracing::debug!(
                        "Got an unhandled message type from LFSWorld Relay? {:?}",
                        data
                    );
                },
            }
        }
    }
}
