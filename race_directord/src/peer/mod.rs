use std::collections::HashMap;
use tokio::{sync::mpsc, task::JoinHandle};
use insim::codec::Frame;

pub (crate) struct PeerManager {
    peers: HashMap<String, JoinHandle<()>>
}

impl PeerManager {
    pub(crate) fn add<F: Frame>(&mut self, id: String, isi: F::Isi) {
        let handle = tokio::task::spawn(async move {

            return ();

        });

        self.peers.insert(id, handle);
    }
    
}
