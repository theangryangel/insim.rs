//! Connection maintains a connection and provides a Stream and Sink of
//! [Packets](crate::packets::Packet).

use async_trait::async_trait;
use if_chain::if_chain;

pub mod tcp;
pub mod traits;
pub mod udp;

pub mod builder;

#[cfg(test)]
mod tests;

use crate::{
    error,
    packets::{
        insim::{Isi, Tiny, TinyType, Version},
        Packet, VERSION,
    },
    result::Result,
};

use insim_core::identifiers::RequestId;
use std::time::Duration;
use tokio::time;

use self::traits::{ReadPacket, ReadWritePacket, WritePacket};

const TIMEOUT_SECS: u64 = 90;

pub struct Connection<I: ReadWritePacket + Send> {
    inner: I,
}

#[async_trait]
impl<I: ReadWritePacket + Send> ReadPacket for Connection<I> {
    async fn read(&mut self) -> Result<Option<Packet>> {
        if let Some(packet) =
            time::timeout(Duration::from_secs(TIMEOUT_SECS), self.inner.read()).await??
        {
            self.maybe_keepalive(&packet).await?;
            return Ok(Some(packet));
        }

        return Ok(None);
    }
}

#[async_trait]
impl<I: ReadWritePacket + Send> WritePacket for Connection<I> {
    async fn write(&mut self, packet: Packet) -> Result<()> {
        self.inner.write(packet).await
    }
}

impl<I: ReadWritePacket + Send> Connection<I> {
    pub fn new(inner: I) -> Self {
        Self { inner }
    }

    pub async fn maybe_keepalive(&mut self, packet: &Packet) -> Result<()> {
        if_chain! {
            if let Packet::Tiny(i) = &packet;
            if i.is_keepalive();
            then {
                let pong = Tiny{
                    subt: TinyType::None,
                    ..Default::default()
                };

                self.write(pong.into()).await?
            }
        }

        Ok(())
    }

    pub async fn handshake(
        &mut self,
        timeout: Duration,
        isi: Isi,
        wait_for_pong: bool,
        verify_version: bool,
    ) -> Result<()> {
        self.inner.write(isi.into()).await?;

        time::timeout(timeout, self.verify(wait_for_pong, verify_version)).await?
    }

    /// Handle the verification of a Transport.
    /// Is Insim server responding the correct version?
    /// Have we received an initial ping response?
    async fn verify(&mut self, verify_version: bool, wait_for_pong: bool) -> Result<()> {
        if wait_for_pong {
            // send a ping!
            self.inner
                .write(
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
            match self.inner.read().await? {
                None => {
                    return Err(error::Error::Disconnected);
                }
                Some(Packet::Tiny(_)) => {
                    received_tiny = true;
                }
                Some(Packet::Version(Version { insimver, .. })) => {
                    if insimver != VERSION {
                        return Err(error::Error::IncompatibleVersion(insimver));
                    }

                    received_vers = true;
                }
                Some(m) => {
                    /* not the droids we're looking for */
                    tracing::info!("received packet whilst waiting for version and/or ping: {m:?}");
                }
            }
        }

        Ok(())
    }
}

impl<I: ReadWritePacket + Send> ReadWritePacket for Connection<I> {}
