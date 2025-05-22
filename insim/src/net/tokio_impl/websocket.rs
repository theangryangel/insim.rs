use core::{
    pin::Pin,
    task::{Context, Poll},
};
use std::io;

use bytes::{Buf, Bytes, BytesMut};
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
        let inner: Bytes = buf.to_vec().into();
        let message = Message::Binary(inner);

        // TODO: Should we be wrapping stream and then polling that?
        match self.inner.poll_ready_unpin(cx) {
            Poll::Ready(Ok(())) => {
                if let Err(e) = self.inner.start_send_unpin(message) {
                    Poll::Ready(Err(io::Error::other(e)))
                } else {
                    let _ = self.poll_flush(cx);
                    Poll::Ready(Ok(buf.len()))
                }
            },
            Poll::Ready(Err(e)) => Poll::Ready(Err(io::Error::other(e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.inner.poll_flush_unpin(cx).map_err(io::Error::other)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        let ws = Pin::new(&mut self.inner);
        ws.poll_close(cx).map_err(io::Error::other)
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
                        return Poll::Ready(Err(io::Error::other(e)));
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
