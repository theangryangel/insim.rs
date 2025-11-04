use std::time::Duration;

use kitcar::{combos::Combo, time::countdown::Countdown};

use crate::{
    Context, GameState,
    combo::ComboExt,
    components::{RootProps, RootScene},
};

pub async fn lobby(cx: Context, combo: Combo<ComboExt>) -> anyhow::Result<Option<GameState>> {
    let mut packets = cx.insim.subscribe();

    let _ = cx.ui.update(RootProps {
        scene: RootScene::Lobby {
            combo: combo.clone(),
            remaining: combo.extensions().restart_after,
        },
    });

    let mut countdown = Countdown::new(
        Duration::from_secs(1),
        cx.config.lobby_duration.as_secs() as u32, // FIXME
    );

    loop {
        tokio::select! {
            remaining = countdown.tick() => match remaining {
                Some(_) => {
                    tracing::info!("Waiting for lobby to complete!");
                    let remaining_duration = countdown.remaining_duration();

                    let _ = cx.ui.update(RootProps {
                        scene: RootScene::Lobby { combo: combo.clone(), remaining: remaining_duration }
                    });
                },
                None => {
                    let _ = cx.ui.update(RootProps {
                        scene: RootScene::Lobby { combo: combo.clone(), remaining: Duration::from_secs(0) }
                    });
                    break;
                }
            },
            packet = packets.recv() => match packet {
                Ok(packet) => {
                    tracing::debug!("PhaseLobby: {:?}", packet);
                },
                _ => {}
            },
            _ = cx.shutdown.cancelled() => {
                return Ok(Some(GameState::Idle));
            }
        }
    }

    Ok(Some(GameState::Round { round: 1, combo }))
}
