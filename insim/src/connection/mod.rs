//! Connection maintains a connection and provides a Stream and Sink of
//! [Packets](crate::packets::Packet).

mod sink;
mod state;
mod stream;

use state::State;

pub mod builder;
pub use builder::ConnectionBuilder;

#[cfg(test)]
mod tests;

use crate::{
    codec::Codec,
    error,
    packets::{
        insim::{Isi, Tiny, TinyType, Version},
        Packet, PacketSinkStream, VERSION,
    },
    result::Result,
};
use pin_project::pin_project;

use futures::{SinkExt, StreamExt};
use insim_core::identifiers::RequestId;
use std::pin::Pin;
use std::task::Context;
use std::time::Duration;
use tokio::time;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_util::codec::Framed;

#[cfg(feature = "udp")]
use crate::udp_stream::UdpStream;

const TIMEOUT_SECS: u64 = 90;

impl<T> PacketSinkStream for Framed<T, Codec> where
    T: AsyncRead + AsyncWrite + std::marker::Unpin + Send
{
}

#[cfg(feature = "tcp")]
pub type TcpConnection = Framed<TcpStream, Codec>;

#[cfg(feature = "udp")]
pub type UdpConnection = Framed<UdpStream, Codec>;

impl<T> PacketSinkStream for Connection<T> where T: PacketSinkStream {}

#[async_trait::async_trait]
pub trait ConnectionTrait: PacketSinkStream {
    /// Request the state of this connection
    fn state(&self) -> State;

    /// Request this client to shutdown
    fn shutdown(&mut self);

    /// Send an Insim Init (ISI) packet, and attempt verification of the connection
    /// live-ness.
    async fn handshake(
        &mut self,
        timeout: Duration,
        isi: Isi,
        wait_for_pong: bool,
        verify_version: bool,
    ) -> Result<()>;
}

/// A Stream and Sink based client for the Insim protocol.
/// Given something that implements [PacketSinkStreamTrait], Connection will handle
/// encoding and decoding of [Packets](Packet), and ensure that the connection
/// is maintained through [Tiny] keepalive packets, and handling any timeout.
#[pin_project]
pub struct Connection<T> {
    #[pin]
    inner: T,
    // Cant use pin_project on tokio::time::Sleep because it's !Unpin
    // meaning we can't then use tokio::select! later. So it needs to be boxed.
    deadline: Pin<Box<time::Sleep>>,
    duration: Duration,
    poll_deadline: bool,
    state: State,
    pong: bool,
}

impl<T> Connection<T>
where
    T: PacketSinkStream + Send,
{
    pub fn new(inner: T) -> Connection<T> {
        let duration = time::Duration::new(TIMEOUT_SECS, 0);
        let next = time::Instant::now() + duration;
        let deadline = Box::pin(tokio::time::sleep_until(next));

        Connection {
            inner,
            deadline,
            duration,
            poll_deadline: true,
            state: State::Connected,
            pong: false,
        }
    }

    pub fn boxed<'a>(self) -> Box<dyn ConnectionTrait + 'a>
    where
        Self: 'a,
    {
        Box::new(self)
    }

    /// Handle the verification of a Transport.
    /// Is Insim server responding the correct version?
    /// Have we received an initial ping response?
    async fn verify(&mut self, verify_version: bool, wait_for_pong: bool) -> Result<()> {
        if wait_for_pong {
            // send a ping!
            //
            <Self as SinkExt<Packet>>::send(
                self,
                Tiny {
                    reqi: RequestId(2),
                    subt: TinyType::Ping,
                }
                .into(),
            )
            .await?;
        }

        let mut received_vers = !verify_version;
        let mut received_tiny = !wait_for_pong;

        while !received_tiny && !received_vers {
            match self.inner.next().await {
                None => {
                    return Err(error::Error::Disconnected);
                }
                Some(Err(e)) => {
                    return Err(e);
                }
                Some(Ok(Packet::Tiny(_))) => {
                    received_tiny = true;
                }
                Some(Ok(Packet::Version(Version { insimver, .. }))) => {
                    if insimver != VERSION {
                        return Err(error::Error::IncompatibleVersion(insimver));
                    }

                    received_vers = true;
                }
                Some(Ok(m)) => {
                    /* not the droids we're looking for */
                    tracing::info!("received packet whilst waiting for version and/or ping: {m:?}");
                }
            }
        }

        Ok(())
    }

    fn poll_pong(mut self: Pin<&mut Self>, cx: &mut Context) {
        if !*self.as_mut().project().pong {
            return;
        }

        tracing::debug!("ping? pong!");

        let res = self.as_mut().project().inner.poll_ready(cx);
        if !res.is_ready() {
            cx.waker().wake_by_ref();
            return;
        }

        let res = self.as_mut().project().inner.start_send(
            Tiny {
                subt: TinyType::None,
                ..Default::default()
            }
            .into(),
        );

        if res.is_err() {
            // TODO: Probably need to fix this.
            cx.waker().wake_by_ref();
            return;
        }

        // TODO: docs suggest we don't need to call poll_flush, but if we don't then nothing gets
        // sent?
        let res = self.as_mut().project().inner.poll_flush(cx);
        if res.is_ready() {
            *self.as_mut().project().pong = false;
        }
    }
}

#[async_trait::async_trait]
impl<T> ConnectionTrait for Connection<T>
where
    T: PacketSinkStream + Send,
{
    fn state(&self) -> State {
        self.state
    }

    fn shutdown(&mut self) {
        self.state = State::Shutdown;
    }

    async fn handshake(
        &mut self,
        timeout: Duration,
        isi: Isi,
        wait_for_pong: bool,
        verify_version: bool,
    ) -> Result<()> {
        self.send(isi).await?;

        time::timeout(timeout, self.verify(wait_for_pong, verify_version)).await??;

        Ok(())
    }
}
