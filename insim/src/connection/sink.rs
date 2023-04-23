use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use futures::Sink;

use super::Connection;
use super::PacketSinkStream;
use crate::packets::Packet;
use crate::result::Result;

impl<T, P> Sink<P> for Connection<T>
where
    T: PacketSinkStream,
    P: Into<Packet>,
{
    type Error = <T as futures::Sink<Packet>>::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, value: P) -> Result<()> {
        let value = value.into();
        tracing::info!("asked to send {value:?}");
        self.project().inner.start_send(value)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_close(cx)
    }
}
