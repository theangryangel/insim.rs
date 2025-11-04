use std::time::Duration;

use kitcar::{combos::Combo, time::countdown::Countdown};

use crate::{
    Context, GameState,
    combo::ComboExt,
    components::{RootProps, RootScene},
};

pub async fn lobby(
    cx: Context,
    combo: Combo<ComboExt>,
    game_id: i64,
) -> anyhow::Result<Option<GameState>> {
    let restart_after = Duration::try_from(cx.config.lobby_duration)?;

    let _ = cx.ui.update(RootProps {
        scene: RootScene::Lobby {
            combo: combo.clone(),
            remaining: restart_after,
        },
    });

    let mut countdown = Countdown::new(
        Duration::from_secs(1),
        restart_after.as_secs() as u32, // FIXME
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
            _ = cx.shutdown.cancelled() => {
                return Ok(None);
            }
        }
    }

    Ok(Some(GameState::Round {
        round: 1,
        combo,
        game_id,
    }))
}
