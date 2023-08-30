//! Connection maintains a connection and provides a Stream and Sink of
//! [Packets](crate::packets::Packet).

use if_chain::if_chain;

pub mod tcp;
pub mod traits;
pub mod udp;

pub mod options;
pub mod transport;

use crate::{
    connection::options::ConnectionOptions,
    error::{self, Error},
    packets::{
        insim::{Isi, Tiny, TinyType, Version},
        Packet, VERSION,
    },
    result::Result,
};

use insim_core::identifiers::RequestId;
use std::time::Duration;
use tokio::{
    sync::{broadcast, mpsc, oneshot},
    time,
};

use self::traits::{ReadPacket, ReadWritePacket, WritePacket};

const TIMEOUT_SECS: u64 = 90;

pub enum Command {
    Send(Packet),
    Firehose(oneshot::Sender<broadcast::Receiver<Packet>>),
    Shutdown,
}

pub struct Connection {
    tx: mpsc::Sender<Command>,
}

impl Connection {
    pub fn new(options: ConnectionOptions) -> Self {
        let (packet_tx, packet_rx) = mpsc::channel(32);

        tokio::spawn(async move { run_actor(&options, packet_rx).await });

        Self { tx: packet_tx }
    }

    pub async fn send<P: Into<Packet>>(&self, packet: P) {
        self.tx
            .send(Command::Send(packet.into()))
            .await
            .expect("Actor task has been killed")
    }

    pub async fn shutdown(&mut self) {
        self.tx
            .send(Command::Shutdown)
            .await
            .expect("Actor task has been killed")
    }

    pub async fn stream(&self) -> broadcast::Receiver<Packet> {
        let (send, recv) = oneshot::channel();
        let msg = Command::Firehose(send);

        // Ignore send errors. If this send fails, so does the
        // recv.await below. There's no reason to check for the
        // same failure twice.
        let _ = self.tx.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }
}

async fn run_actor(
    options: &ConnectionOptions,
    mut rx: mpsc::Receiver<Command>,
) -> crate::result::Result<()> {
    let mut attempt: u64 = 0;

    loop {
        let (broadcast_rx, _) = broadcast::channel(32);

        // connect
        let isi = options.as_isi();
        let mut stream = match options.transport.connect(isi).await {
            Ok(stream) => {
                attempt = 0;
                stream
            }
            Err(_) => {
                (attempt, _) = attempt.overflowing_add(1);
                match options.reconnect.retry(&attempt) {
                    (true, Some(delay)) => {
                        time::sleep(delay).await;
                        continue;
                    }
                    (true, None) => {
                        continue;
                    }
                    _ => {
                        return Err(Error::Disconnected);
                    }
                }
            }
        };

        // TODO: Communicate initial connection state
        // TODO: Communicate connection failures and retries
        // TODO: Tests.

        loop {
            tokio::select! {

                res = rx.recv() => match res {
                    Some(Command::Send(packet)) => {
                        stream.write(packet).await?;
                    },
                    Some(Command::Firehose(respond_to)) => {
                        let _ = respond_to.send(broadcast_rx.subscribe());
                    },
                    Some(Command::Shutdown) => return Ok(()),
                    None => break,
                },

                res = time::timeout(
                    Duration::from_secs(TIMEOUT_SECS), stream.read()
                ) => {

                    // FIXME we canot just ?? we need to handle the disconnects, etc.
                    match res?? {
                        Some(packet) => {
                            maybe_keepalive(&mut stream, &packet).await?;

                            broadcast_rx.send(packet).expect("????");
                        },
                        None => {
                            return Ok(());
                        },
                    }
                }

            }
        }
    }
}

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

/// Handle the verification of a Transport.
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
