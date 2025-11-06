use std::{collections::HashMap, time::Duration};

use insim::{Packet, identifiers::ConnectionId};
use kitcar::{combos::Combo, time::countdown::Countdown};
use tokio::time::sleep;

use crate::{
    Context, Scene,
    combo::ComboExt,
    components::{RootProps, RootScene},
};

#[derive(Debug, Clone)]
pub struct Round {
    pub game_id: i64,
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
        let mut round_scores: HashMap<String, Duration> = HashMap::new();

        cx.insim.send_command("/restart").await?;

        cx.ui.update(RootProps {
            scene: RootScene::Round {
                round: self.round,
                combo: self.combo.clone(),
                remaining: available_time.into(),
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

        let scores_by_position = &cx.config.scores_by_position;

        // score round
        let mut ordered = round_scores
            .drain()
            .map(|(k, v)| (k, target.abs_diff(v)))
            .collect::<Vec<(String, Duration)>>();

        ordered.sort_by(|a, b| a.1.cmp(&b.1));

        // FIXME: this should be batched so we can do it as a single transaction
        for (i, (uname, delta)) in ordered
            .into_iter()
            .take(scores_by_position.len())
            .enumerate()
        {
            let points = scores_by_position[i];
            let _ = cx.leaderboard.add_score(uname.clone(), points as i32).await;
            // XXX: This truncates the delta, but realistically nothing should be this high
            cx.database.insert_player_score(
                self.game_id,
                self.round,
                &uname,
                points,
                i,
                delta.as_millis() as u64,
            )?;
            tracing::info!(
                "Player {} scored {} points (delta: {:?})",
                uname,
                points,
                delta
            );
        }

        cx.insim
            .send_message(
                &format!("Round {} complete!", self.round),
                ConnectionId::ALL,
            )
            .await?;

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
    game_id: i64,
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
