use std::time::Duration;

use kitcar::{combos::Combo, time::countdown::Countdown};

use crate::{
    Context, Scene,
    combo::ComboExt,
    components::{RootProps, RootScene},
};

#[derive(Debug, Clone)]
pub struct Lobby {
    pub combo: Combo<ComboExt>,
    pub game_id: i64,
}

impl Lobby {
    pub async fn run(self, cx: Context) -> anyhow::Result<Option<Scene>> {
        let restart_after = Duration::try_from(cx.config.lobby_duration)?;

        cx.ui.update(RootProps {
            scene: RootScene::Lobby {
                combo: self.combo.clone(),
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

                        cx.ui.update(RootProps {
                            scene: RootScene::Lobby { combo: self.combo.clone(), remaining: remaining_duration }
                        });
                    },
                    None => {
                        cx.ui.update(RootProps {
                            scene: RootScene::Lobby { combo: self.combo.clone(), remaining: Duration::from_secs(0) }
                        });
                        break;
                    }
                },
                _ = cx.shutdown.cancelled() => {
                    return Ok(None);
                }
            }
        }

        Ok(Some(
            super::Round {
                round: 1,
                combo: self.combo,
                game_id: self.game_id,
            }
            .into(),
        ))
    }
}
