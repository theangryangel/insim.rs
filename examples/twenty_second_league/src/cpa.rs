use insim::Packet;
use kitcar::{plugin::UserState, PluginContext};

pub(crate) async fn cpa<S: UserState>(ctx: PluginContext<S>) -> Result<(), ()> {
    let mut packets = ctx.subscribe_to_packets();

    while let Ok(packet) = packets.recv().await {
        if_chain::if_chain! {
            if let Packet::Pen(pen) = packet;
            if let Some(player) = ctx.get_player(pen.plid).await;
            if let Some(connection) = ctx.get_connection(player.ucid).await;
            if connection.uname.len() > 0;
            then {
                ctx.send_command(
                    &format!("/p_clear {}", &connection.uname),
                ).await;
            }
        }
    }

    Ok(())
}
