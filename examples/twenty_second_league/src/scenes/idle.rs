use insim::Packet;
use kitcar::chat::Parse;

use crate::{
    Context, MyChatCommands, Scene,
    components::{RootProps, RootScene},
};

#[derive(Debug, Clone)]
pub struct Idle;

impl Idle {
    pub async fn run(self, cx: Context) -> anyhow::Result<Option<Scene>> {
        cx.leaderboard.clear().await;

        cx.ui.update(RootProps {
            scene: RootScene::Idle,
        });

        let mut packets = cx.insim.subscribe();

        loop {
            tokio::select! {
                _ = cx.shutdown.cancelled() => {
                    return Ok(None);
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
                                    let game_id = cx.database.new_game(&combo.extensions().name)?;

                                    return Ok(Some(super::TrackRotation { combo: combo.clone(), game_id }.into()));
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
    }
}
