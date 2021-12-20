use super::protocol;

use tokio::sync::mpsc;

#[derive(Clone)]
pub struct Ctx<State> {
    pub state: State,

    pub(crate) shutdown: Option<mpsc::UnboundedSender<bool>>,
    pub(crate) tx: Option<mpsc::UnboundedSender<protocol::Packet>>,
}

impl<State> Ctx<State> {
    /// Send a [Packet](super::protocol::Packet).
    #[allow(unused_must_use)] // if this fails then the we're probably going to die anyway
    pub fn send(&self, data: protocol::Packet) {
        if let Some(tx) = &self.tx {
            tx.send(data);
        }
    }

    /// Request shutdown of the client.
    #[allow(unused_must_use)] // if this fails then the we're probably going to die anyway
    pub fn shutdown(&self) {
        if let Some(shutdown) = &self.shutdown {
            shutdown.send(true);
        }
    }
}
