use std::{collections::HashMap, time::Duration};

use insim::{Packet, identifiers::ConnectionId};
use kitcar::{combos::Combo, time::countdown::Countdown};
use tokio::time::sleep;

use crate::{
    Context, Scene,
    combo::ComboExt,
    components::{RootProps, RootScene},
    db::game::EventId,
};

#[derive(Debug, Clone)]
pub struct Round {
    pub game_id: EventId,
    pub round: u32,
    pub combo: Combo<ComboExt>,
}

impl Round {
    pub async fn run(self, cx: Context) -> anyhow::Result<Option<Scene>> {
        if cx.presence.player_count().await < 1 {
            cx.insim
                .send_message(
                    "Returning to idle state because there's no players remaining",
                    ConnectionId::ALL,
                )
                .await?;
            tracing::info!("Returning to idle state because there's no players remaining");
            return Ok(Some(super::Idle.into()));
        }

        let mut packets = cx.insim.subscribe();

        let target = Duration::try_from(self.combo.extensions().target_time)?;
        let rounds = self.combo.extensions().rounds;
        let available_time = Duration::try_from(self.combo.extensions().restart_after)?;
        let mut round_scores: HashMap<String, Duration> = HashMap::new(); // TODO: Only the last scoring round is stored. add to MOTD

        cx.insim.send_command("/restart").await?;

        let scores = cx.database.leaderboard(self.game_id, 10)?;

        cx.ui.update(RootProps {
            scene: RootScene::Round {
                round: self.round,
                combo: self.combo.clone(),
                remaining: available_time.into(),
                scores: scores.clone(),
            },
        });

        tracing::info!("Starting round {}/{}", self.round, rounds);
        tracing::debug!("Waiting for game to start");

        // TODO: how do we prevent this from causing issues
        // its convenient. do we care?
        cx.game.wait_for_racing().await;
        sleep(Duration::from_secs(11)).await;

        cx.insim
            .send_message(
                &format!(
                    "Round {}/{} - Get close to {}!",
                    self.round,
                    rounds,
                    self.combo.extensions().target_time
                ),
                ConnectionId::ALL,
            )
            .await?;

        let mut countdown = Countdown::new(Duration::from_secs(1), available_time.as_secs() as u32);

        loop {
            tokio::select! {
                remaining = countdown.tick() => match remaining {
                    Some(_) => {
                        let remaining_duration = countdown.remaining_duration();
                        tracing::debug!("{:?}s remaining!", &remaining_duration);

                        cx.ui.update(RootProps {
                            scene: RootScene::Round {
                                round: self.round,
                                combo: self.combo.clone(),
                                remaining: remaining_duration,
                                scores: scores.clone(),
                            }
                        });
                    },
                    None => {
                        break;
                    }
                },
                packet = packets.recv() => match packet? {
                    Packet::Fin(fin) => {
                        let conn_info = cx.presence.connection_by_player(&fin.plid).await;
                        if let Some(conn_info) = conn_info {
                            let _ = round_scores.insert(conn_info.uname, fin.ttime);
                        }
                    },
                    Packet::Ncn(ncn) => {
                        cx.insim
                            .send_message(
                                "Welcome to the Cadence Cup! Game in currently in progress!",
                                ncn.ucid,
                            )
                            .await?;
                    },
                    _ => {},
                },
                _ = cx.shutdown.cancelled() => {
                    return Ok(None);
                }
            }
        }

        // let the scoring complete atomically, if without checking the cancel / shutdown token - this
        // is by design.

        let max_scorers = cx.config.max_scoring_players;

        // score round
        let mut ordered = round_scores
            .drain()
            .map(|(k, v)| (k, target.abs_diff(v)))
            .collect::<Vec<(String, Duration)>>();
        ordered.sort_by(|a, b| a.1.cmp(&b.1));
        let top: Vec<(String, i32, usize, Duration)> = ordered
            .into_iter()
            .take(max_scorers)
            .enumerate()
            .map(|(i, (uname, delta))| {
                let points = (max_scorers - i) as i32;
                (uname, points, i, delta)
            })
            .collect();

        cx.database
            .insert_player_scores(self.game_id, self.round, top)?;

        cx.insim
            .send_message(
                &format!("Round {} complete!", self.round),
                ConnectionId::ALL,
            )
            .await?;

        let scores = cx.database.leaderboard(self.game_id, 10)?;

        cx.ui.update(RootProps {
            scene: RootScene::Round {
                round: self.round,
                combo: self.combo.clone(),
                remaining: Duration::ZERO,
                scores: scores.clone(),
            },
        });

        if cx.shutdown.is_cancelled() {
            Ok(None)
        } else if self.round + 1 > self.combo.extensions().rounds {
            // TODO: send leaderboard
            Ok(Some(
                Victory {
                    game_id: self.game_id,
                }
                .into(),
            ))
        } else {
            Ok(Some(
                Round {
                    round: self.round + 1,
                    combo: self.combo,
                    game_id: self.game_id,
                }
                .into(),
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Victory {
    game_id: EventId,
}

impl Victory {
    pub async fn run(self, cx: Context) -> anyhow::Result<Option<Scene>> {
        let duration = Duration::try_from(cx.config.victory_duration)?;

        cx.ui.update(RootProps {
            scene: RootScene::Victory {
                remaining: duration,
            },
        });

        let mut countdown = Countdown::new(Duration::from_secs(1), duration.as_secs() as u32);

        while let Some(_) = countdown.tick().await {
            let remaining = countdown.remaining_duration();

            cx.ui.update(RootProps {
                scene: RootScene::Victory { remaining },
            });
        }

        cx.database.complete_event(self.game_id)?;

        Ok(Some(super::Idle.into()))
    }
}
