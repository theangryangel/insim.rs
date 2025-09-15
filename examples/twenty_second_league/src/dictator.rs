//! No voting!

use std::fmt::Debug;
use insim::{insim::TinyType, Packet};
use kitcar::PluginContext;

pub(crate) async fn dictator<S: Send + Sync + Clone + Debug + 'static>(ctx: PluginContext<S>) -> Result<(), ()> {
    let mut packets = ctx.subscribe_to_packets();

    while let Ok(packet) = packets.recv().await {
        if matches!(packet, Packet::Vtn(_)) {
            ctx.send_packet(TinyType::Vtc).await;
        }
    }

    Ok(())
}
