use bytes::BytesMut;
use futures::{Sink, Stream};
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::net::TcpStream;

use crate::{
    core::{Decodable, Encodable},
    packets::Packet,
    packets::PacketSinkStream,
};

/// A wrapper around tokio::net::UdpSocket to make it easier to work with.
#[pin_project]
pub struct WebSocketClientTransport {
    #[pin]
    inner: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
}

impl PacketSinkStream for WebSocketClientTransport {}

impl WebSocketClientTransport {
    pub async fn connect(remote_addr: url::Url) -> tokio::io::Result<Self> {
        // FIXME handle the error properly!
        let (inner, _) = tokio_tungstenite::connect_async(remote_addr).await.unwrap();

        Ok(Self { inner })
    }
}

impl Stream for WebSocketClientTransport {
    type Item = crate::result::Result<crate::packets::Packet>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(a)))) => {
                let mut buf = BytesMut::new();
                buf.extend_from_slice(&a);
                let data = Packet::decode(&mut buf, None)?;

                Poll::Ready(Some(Ok(data)))
            }

            Poll::Ready(Some(Ok(_))) => {
                tracing::debug!("ignoring non-binary message!");
                Poll::Pending
            }

            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e.into()))),
        }
    }
}

impl<I> Sink<I> for WebSocketClientTransport
where
    I: Into<crate::packets::Packet>,
{
    type Error = crate::error::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx).map_err(|e| e.into())
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        let item: Packet = item.into();

        let mut data = BytesMut::new();
        item.encode(&mut data, None)?;

        self.project()
            .inner
            .start_send(tokio_tungstenite::tungstenite::Message::Binary(
                data.to_vec(),
            ))
            .map_err(|e| e.into())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx).map_err(|e| e.into())
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx).map_err(|e| e.into())
    }
}
