use insim::Packet;
use kitcar::chat::Parse;

use crate::{
    Context, GameState, MyChatCommands,
    components::{RootProps, RootScene},
};

pub async fn idle(cx: Context) -> anyhow::Result<Option<GameState>> {
    cx.leaderboard.clear().await;

    let _ = cx.ui.update(RootProps {
        scene: RootScene::Idle,
    });

    let mut packets = cx.insim.subscribe();

    loop {
        tokio::select! {
            _ = cx.shutdown.cancelled() => {
                break;
            },
            packet = packets.recv() => match packet? {
                Packet::Ncn(ncn) if !ncn.ucid.local() => {
                    cx.insim
                        .send_message(
                            &format!("Welcome. No game is currently in progress."),
                            ncn.ucid,
                        )
                        .await?;
                },
                Packet::Mso(mso) => {
                    if_chain::if_chain! {
                        if let Ok(MyChatCommands::Start) = MyChatCommands::parse(mso.msg_from_textstart());
                        if let Some(conn_info) = cx.presence.connection(&mso.ucid).await;
                        if conn_info.admin;
                        then {
                            if let Some(combo) = cx.config.combos.random() {
                                return Ok(Some(GameState::TrackRotation { combo: combo.clone() }));
                            } else {
                                cx.insim.send_message("No configured combos founded", conn_info.ucid).await?;
                            }
                        }
                    }
                },
                _ => {},
            }
        }
    }

    Ok(Some(GameState::Idle))
}
