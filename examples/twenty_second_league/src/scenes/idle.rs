use insim::{identifiers::PlayerId, Packet};
use kitcar::{combos::ComboList, leaderboard::LeaderboardHandle, presence::PresenceHandle};

use crate::{combo::ComboExt, GameState, MyChatCommands};

pub async fn idle(
    insim: insim::builder::SpawnedHandle,
    presence: PresenceHandle,
    leaderboard: LeaderboardHandle<PlayerId>,
    combos: ComboList<ComboExt>,
) -> anyhow::Result<GameState> {
    leaderboard.clear().await;

    let mut packets = insim.subscribe();

    while let Ok(packet) = packets.recv().await {
        match packet {
            Packet::Ncn(ncn) if !ncn.ucid.local() => {
                insim
                    .send_message(
                        &format!("Welcome. No game is currently in progress."),
                        ncn.ucid,
                    )
                    .await?;
            },
            Packet::Mso(mso) => {
                if_chain::if_chain! {
                    if let Ok(MyChatCommands::Start) = MyChatCommands::parse_with_prefix(mso.msg_from_textstart(), Some('!'));
                    if let Some(conn_info) = presence.connection(&mso.ucid).await;
                    if conn_info.admin;
                    then {
                        if let Some(combo) = combos.random() {
                            return Ok(GameState::TrackRotation { combo: combo.clone() });
                        } else {
                            insim.send_message("No configured combos founded", conn_info.ucid).await?;
                        }
                    }
                }
            },
            _ => {},
        }
    }

    Ok(GameState::Idle)
}
