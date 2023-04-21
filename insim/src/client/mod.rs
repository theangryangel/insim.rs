//! Insim Client that maintains a connection and provides a Stream and Sink of
//! [Packets](crate::packets::Packet).

mod sink;
mod state;
mod stream;

use state::ClientState;

pub mod builder;
pub use builder::ClientBuilder;

#[cfg(test)]
mod tests;

use crate::{
    codec::Codec,
    error,
    packets::{
        insim::{self, Tiny, TinyType},
        Packet,
    },
    result::Result,
};
use futures::{Sink, Stream};
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

#[cfg(feature = "game_state")]
use crate::game_state::GameState;

const TIMEOUT_SECS: u64 = 90;

pub trait PacketSinkStreamTrait:
    Sink<Packet, Error = error::Error> + Stream<Item = Result<Packet>> + std::marker::Unpin + Send
{
}
impl<T> PacketSinkStreamTrait for Framed<T, Codec> where
    T: AsyncRead + AsyncWrite + std::marker::Unpin + Send
{
}

#[cfg(feature = "tcp")]
pub type TcpClientTransport = Framed<TcpStream, Codec>;

#[cfg(feature = "udp")]
pub type UdpClientTransport = Framed<UdpStream, Codec>;

impl<T> PacketSinkStreamTrait for Client<T> where T: PacketSinkStreamTrait {}

#[async_trait::async_trait]
pub trait ClientTrait: PacketSinkStreamTrait {
    /// Request the ClientState
    fn state(&self) -> ClientState;

    /// Request this client to shutdown
    fn shutdown(&mut self);

    /// Send an Insim Init (ISI) packet, and attempt verification of the connection
    /// live-ness.
    async fn handshake(
        &mut self,
        timeout: Duration,
        isi: crate::packets::insim::Isi,
        wait_for_pong: bool,
        verify_version: bool,
    ) -> Result<()>;

    #[cfg(feature = "game_state")]
    async fn request_game_state(&mut self) -> Result<()>;

    #[cfg(feature = "game_state")]
    fn get_players(&self) -> Vec<crate::game_state::connection::Connection>;

    #[cfg(feature = "game_state")]
    fn get_connections(&self) -> Vec<crate::game_state::connection::Connection>;
}

/// A Stream and Sink based client for the Insim protocol.
/// Given something that implements the [ClientTransport] trait, Client will handle
/// encoding and decoding of [Packets](Packet), and ensure that the connection
/// is maintained through [insim::Tiny] keepalive packets, and handling any timeout.
#[pin_project]
pub struct Client<T> {
    #[pin]
    inner: T,
    // Cant use pin_project on tokio::time::Sleep because it's !Unpin
    // meaning we can't then use tokio::select! later. So it needs to be boxed.
    deadline: Pin<Box<time::Sleep>>,
    duration: Duration,
    poll_deadline: bool,
    state: ClientState,
    pong: bool,

    #[cfg(feature = "game_state")]
    pub game: GameState,
}

impl<T> Client<T>
where
    T: PacketSinkStreamTrait + Send,
{
    pub fn new(inner: T) -> Client<T> {
        let duration = time::Duration::new(TIMEOUT_SECS, 0);
        let next = time::Instant::now() + duration;
        let deadline = Box::pin(tokio::time::sleep_until(next));

        Client {
            inner,
            deadline,
            duration,
            poll_deadline: true,
            state: ClientState::Connected,
            pong: false,

            #[cfg(feature = "game_state")]
            game: GameState::new(),
        }
    }

    pub fn boxed<'a>(self) -> Box<dyn ClientTrait + 'a>
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
                    subtype: TinyType::Ping,
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
                Some(Ok(Packet::Version(insim::Version { insimver, .. }))) => {
                    if insimver != crate::packets::VERSION {
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
                subtype: TinyType::None,
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
impl<T> ClientTrait for Client<T>
where
    T: PacketSinkStreamTrait + Send,
{
    fn state(&self) -> ClientState {
        self.state
    }

    fn shutdown(&mut self) {
        self.state = ClientState::Shutdown;
    }

    async fn handshake(
        &mut self,
        timeout: Duration,
        isi: crate::packets::insim::Isi,
        wait_for_pong: bool,
        verify_version: bool,
    ) -> Result<()> {
        self.send(isi).await?;

        time::timeout(timeout, self.verify(wait_for_pong, verify_version)).await??;

        Ok(())
    }

    #[cfg(feature = "game_state")]
    async fn request_game_state(&mut self) -> Result<()> {
        let todo = [TinyType::Ncn, TinyType::Npl];

        for (i, p) in todo.iter().enumerate() {
            <Self as SinkExt<Packet>>::send(
                self,
                Tiny {
                    subtype: p.clone(),
                    reqi: RequestId(i as u8),
                }
                .into(),
            )
            .await?;
        }

        Ok(())
    }

    #[cfg(feature = "game_state")]
    fn get_players(&self) -> Vec<crate::game_state::connection::Connection> {
        self.game.get_players()
    }

    #[cfg(feature = "game_state")]
    fn get_connections(&self) -> Vec<crate::game_state::connection::Connection> {
        self.game.get_connections()
    }
}
