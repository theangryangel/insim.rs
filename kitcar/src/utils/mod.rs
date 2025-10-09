//! Misc utilies

use insim::{insim::TinyType, Packet};

use crate::Service;

#[derive(Debug)]
/// No Voting
pub struct NoVote;

impl Service for NoVote {
    fn spawn(insim: insim::builder::SpawnedHandle) {
        let _ = tokio::spawn(async move {
            let mut packet_rx = insim.subscribe();
            while let Ok(packet) = packet_rx.recv().await {
                if matches!(packet, Packet::Vtn(_)) {
                    let _ = insim.send(TinyType::Vtc).await;
                }
            }
        });
    }
}
