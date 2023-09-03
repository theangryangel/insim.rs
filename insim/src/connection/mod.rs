//! Connection maintains a connection and provides a Stream and Sink of
//! [Packets](crate::packets::Packet).

mod options;
mod r#type;

pub use options::{ConnectionOptions, ReconnectOptions};

use crate::{
    error::Error,
    packets::Packet,
    result::Result,
    tools::{handshake, maybe_keepalive},
    traits::{ReadPacket, WritePacket},
};

use std::time::Duration;

use tokio::{
    sync::{broadcast, mpsc, oneshot},
    time,
};

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

async fn run_actor(options: &ConnectionOptions, mut rx: mpsc::Receiver<Command>) -> Result<()> {
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
