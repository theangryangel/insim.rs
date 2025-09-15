use insim::{insim::Mst, Packet};
use kitcar::PluginContext;
use std::fmt::Debug;

pub(crate) async fn cpa<S: Send + Sync + Clone + Debug + 'static>(ctx: PluginContext<S>) -> Result<(), ()> {
    let mut packets = ctx.subscribe_to_packets();

    while let Ok(packet) = packets.recv().await {
        if_chain::if_chain! {
            if let Packet::Pen(pen) = packet;
            if let Some(player) = ctx.get_player(pen.plid).await;
            if let Some(connection) = ctx.get_connection(player.ucid).await;
            if connection.uname.len() > 0;
            then {
                ctx.send_packet(Mst {
                    msg: format!("/p_clear {}", &connection.uname),
                    ..Default::default()
                }).await;
            }
        }
    }

    Ok(())
}
