use std::time::Duration;

use insim::{Packet, core::track::Track, identifiers::ConnectionId};
use tokio::time::timeout;

use crate::{GameState, MyChatCommands, MyContext};

pub async fn idle(
    insim: insim::builder::SpawnedHandle,
    state: MyContext,
) -> anyhow::Result<GameState> {
    let mut packets = insim.subscribe();

    loop {
        let packet = packets.recv().await?;

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
                    if let Some(conn_info) = state.presence.connection(&mso.ucid).await;
                    if conn_info.admin;
                    then {
                        let result = timeout(Duration::from_secs(60), async {
                            println!("Transitioning to game");

                            let _ = insim.send_command("/end").await;
                            let _ = insim.send_message("Waiting for track selection screen...", ConnectionId::ALL);
                            state.game.wait_for_end().await;

                            let _ = insim.send_message("Requesting track change", ConnectionId::ALL).await;
                            let _ = insim.send_command("/track FE1").await;

                            let _ = insim.send_message("Waiting for track", ConnectionId::ALL).await;
                            state.game.wait_for_track(Track::Fe1).await;

                            let _ = insim.send_message("Waiting for game to start", ConnectionId::ALL).await;
                            state.game.wait_for_racing().await;
                        }).await;

                        if result.is_ok() {
                            return Ok(GameState::Lobby)
                        } else {
                            insim.send_message("Failed to start game after 60 seconds.. going idle", ConnectionId::ALL).await?;
                            return Ok(GameState::Idle)
                        }

                    }
                }
            },
            _ => {},
        }
    }
}
