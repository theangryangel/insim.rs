//! Connection maintains a connection and provides a Stream and Sink of
//! [Packets](crate::packets::Packet).

mod actor;
mod command;
mod event;
mod options;
mod r#type;

use command::Command;
pub use event::Event;

pub use options::{ConnectionOptions, ReconnectOptions};

use crate::{packets::Packet, tools::handshake};

use tokio::sync::{broadcast, mpsc, oneshot};

pub struct Connection {
    tx: mpsc::Sender<Command>,
}

impl Connection {
    pub fn new(options: ConnectionOptions) -> Self {
        let (packet_tx, packet_rx) = mpsc::channel(32);

        tokio::spawn(async move { actor::run_actor(&options, packet_rx).await });

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

    pub async fn stream(&self) -> broadcast::Receiver<Event> {
        let (send, recv) = oneshot::channel();
        let msg = Command::Firehose(send);

        // Ignore send errors. If this send fails, so does the
        // recv.await below. There's no reason to check for the
        // same failure twice.
        let _ = self.tx.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }
}
