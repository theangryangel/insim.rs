use std::{collections::HashMap, time::Duration};

use insim::{Packet, identifiers::ConnectionId};
use kitcar::{combos::Combo, time::countdown::Countdown};
use tokio::time::sleep;

use crate::{
    Context, GameState,
    combo::ComboExt,
    components::{RootProps, RootScene},
};

pub async fn round(
    cx: Context,
    round: u32,
    combo: Combo<ComboExt>,
) -> anyhow::Result<Option<GameState>> {
    let mut packets = cx.insim.subscribe();

    let target = combo.extensions().target_time;
    let rounds = combo.extensions().rounds;
    let available_time = combo.extensions().restart_after;
    let mut round_scores: HashMap<String, Duration> = HashMap::new();

    cx.insim.send_command("/restart").await?;

    let _ = cx.ui.update(RootProps {
        scene: RootScene::Round {
            round,
            combo: combo.clone(),
            remaining: available_time.into(),
        },
    });

    tracing::info!("Starting round {}/{}", round, rounds);
    tracing::debug!("Waiting for game to start");

    // TODO: how do we prevent this from causing issues
    // its convenient. do we care?
    cx.game.wait_for_racing().await;
    sleep(Duration::from_secs(11)).await;

    cx.insim
        .send_message(
            &format!("Round {}/{} - Get close to 20s!", round, rounds),
            ConnectionId::ALL,
        )
        .await
        .unwrap();

    let mut countdown = Countdown::new(Duration::from_secs(1), available_time.as_secs() as u32);

    loop {
        tokio::select! {
            remaining = countdown.tick() => match remaining {
                Some(_) => {
                    let remaining_duration = countdown.remaining_duration();
                    tracing::debug!("{:?}s remaining!", &remaining_duration);

                    let _ = cx.ui.update(RootProps {
                        scene: RootScene::Round {
                            round,
                            combo: combo.clone(),
                            remaining: remaining_duration,
                        }
                    });
                },
                None => {
                    break;
                }
            },
            packet = packets.recv() => match packet.unwrap() {
                Packet::Fin(fin) => {
                    let conn_info = cx.presence.connection_by_player(&fin.plid).await;
                    if let Some(conn_info) = conn_info {
                        let _ = round_scores.insert(conn_info.uname, fin.ttime);
                    }
                },
                Packet::Ncn(ncn) => {
                    cx.insim
                        .send_message(
                            "Welcome to 20 Second League! Get as close to 20s as possible.",
                            ncn.ucid,
                        )
                        .await
                        .unwrap();
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

    for (i, (uname, delta)) in ordered
        .into_iter()
        .take(scores_by_position.len())
        .enumerate()
    {
        let points = scores_by_position[i];
        let _ = cx.leaderboard.add_score(uname.clone(), points as i32).await;
        tracing::info!(
            "Player {} scored {} points (delta: {:?})",
            uname,
            points,
            delta
        );
    }

    cx.insim
        .send_message(&format!("Round {} complete!", round), ConnectionId::ALL)
        .await
        .unwrap();

    if cx.shutdown.is_cancelled() {
        Ok(None)
    } else if round + 1 > combo.extensions().rounds {
        // TODO: send leaderboard
        Ok(Some(GameState::Victory))
    } else {
        Ok(Some(GameState::Round {
            round: round + 1,
            combo,
        }))
    }
}

pub async fn victory(cx: Context) -> anyhow::Result<Option<GameState>> {
    cx.ui.update(RootProps {
        scene: RootScene::Victory {
            remaining: cx.config.victory_duration,
        },
    });

    let mut countdown = Countdown::new(
        Duration::from_secs(1),
        cx.config.victory_duration.as_secs() as u32,
    );

    while let Some(_) = countdown.tick().await {
        let remaining = countdown.remaining_duration();

        let _ = cx.ui.update(RootProps {
            scene: RootScene::Victory { remaining },
        });
    }

    Ok(Some(GameState::Idle))
}
