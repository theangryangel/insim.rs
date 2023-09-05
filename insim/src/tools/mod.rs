use if_chain::if_chain;
use insim_core::identifiers::RequestId;
use std::time::Duration;
use tokio::time;

use crate::{
    error::Error,
    packets::{
        insim::{Isi, Tiny, TinyType, Version},
        Packet, VERSION,
    },
    result::Result,
    traits::ReadWritePacket,
};

pub async fn handshake<I: ReadWritePacket>(
    inner: &mut I,
    timeout: Duration,
    isi: Isi,
    wait_for_pong: bool,
    verify_version: bool,
) -> Result<()> {
    time::timeout(timeout, inner.write(isi.into())).await??;

    time::timeout(timeout, verify(inner, wait_for_pong, verify_version)).await?
}

/// Handle the verification of anything that implements
/// ReadWritePacket.
/// Is Insim server responding the correct version?
/// Have we received an initial ping response?
async fn verify<I: ReadWritePacket>(
    inner: &mut I,
    verify_version: bool,
    wait_for_pong: bool,
) -> Result<()> {
    if wait_for_pong {
        // send a ping!
        inner
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
        match inner.read().await? {
            Packet::Tiny(_) => {
                received_tiny = true;
            }
            Packet::Version(Version { insimver, .. }) => {
                if insimver != VERSION {
                    return Err(Error::IncompatibleVersion(insimver));
                }

                received_vers = true;
            }
            packet => {
                /* not the droids we're looking for */
                tracing::debug!(
                    "received packet whilst waiting for version and/or ping: {packet:?}"
                );
            }
        }
    }

    Ok(())
}

pub async fn maybe_keepalive<I: ReadWritePacket>(inner: &mut I, packet: &Packet) -> Result<()> {
    if_chain! {
        if let Packet::Tiny(i) = &packet;
        if i.is_keepalive();
        then {
            let pong = Tiny{
                subt: TinyType::None,
                ..Default::default()
            };

            inner.write(pong.into()).await?
        }
    }

    Ok(())
}
