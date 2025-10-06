//! Misc utilies

use insim::{insim::{TinyType}, Packet};
use tokio::sync::{broadcast, mpsc};

use crate::Service;

#[derive(Debug)]
/// No Voting
pub struct NoVote;

impl Service for NoVote {
    fn spawn(mut packet_rx: broadcast::Receiver<insim::Packet>, packet_tx: mpsc::Sender<insim::Packet>) {
        let _ = tokio::spawn(async move {
            while let Ok(packet) = packet_rx.recv().await {
                if matches!(packet, Packet::Vtn(_)) {
                    let _ = packet_tx.send(TinyType::Vtc.into()).await;
                }
            }
        });
    }
}
