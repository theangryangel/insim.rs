use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

use bytes::{Bytes, BytesMut};

use crate::MAX_SIZE_PACKET;

pub(crate) type TungsteniteWebSocket =
    tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
/// Websocket "stream" wrapper.
pub struct WebsocketStream {
    inner: TungsteniteWebSocket,
    buf: BytesMut,
}

impl From<TungsteniteWebSocket> for WebsocketStream {
    fn from(value: TungsteniteWebSocket) -> Self {
        Self {
            inner: value,
            buf: BytesMut::with_capacity(MAX_SIZE_PACKET),
        }
    }
}

impl Read for WebsocketStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // If we have no buffered data, read the next message
        while self.buf.is_empty() {
            match self.inner.read() {
                Ok(msg) => match msg {
                    tungstenite::Message::Binary(data) => {
                        self.buf.extend_from_slice(&data);
                    },
                    tungstenite::Message::Close(_) => return Ok(0),
                    _ => continue,
                },
                Err(tungstenite::Error::Io(e)) => return Err(e),
                Err(e) => return Err(io::Error::other(e)),
            }
        }

        let to_copy = self.buf.len().min(buf.len());
        buf[..to_copy].copy_from_slice(&self.buf.split_to(to_copy));
        Ok(to_copy)
    }
}

impl Write for WebsocketStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let message: Bytes = buf.to_vec().into();
        self.inner
            .write(tungstenite::Message::Binary(message))
            .map_err(io::Error::other)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush().map_err(io::Error::other)
    }
}
