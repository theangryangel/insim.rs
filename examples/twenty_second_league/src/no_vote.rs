//! No voting!

use insim::{Packet, insim::TinyType};

#[kitcar::service]
pub fn NoVote() {
    let _handle = tokio::spawn(async move {
        let mut packet_rx = insim.subscribe();
        while let Ok(packet) = packet_rx.recv().await {
            if matches!(packet, Packet::Vtn(_)) {
                let _ = insim.send(TinyType::Vtc).await;
            }
        }
    });
}
