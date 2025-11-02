use std::{collections::HashMap, time::Duration};

use insim::{
    Packet,
    identifiers::{ConnectionId, PlayerId},
};
use kitcar::{combos::Combo, time::countdown::Countdown};
use tokio::time::sleep;

use crate::{
    Context, GameState,
    combo::ComboExt,
    components::{RootProps, RootScene},
};

pub async fn round(cx: Context, round: u32, combo: Combo<ComboExt>) -> anyhow::Result<GameState> {
    let mut packets = cx.insim.subscribe();

    let target = combo.extensions().target_time;
    let rounds = combo.extensions().rounds;
    let available_time = combo.extensions().restart_after;
    let mut round_scores: HashMap<PlayerId, Duration> = HashMap::new();

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
                    let _ = round_scores.insert(fin.plid, fin.ttime);
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
                Packet::Pll(pll) => {
                    // FIXME: probably unfair, but fuck it for now
                    let _ = round_scores.remove(&pll.plid);
                },
                _ => {},
            }
        }
    }

    let scores_by_position = &cx.config.scores_by_position;

    // score round
    let mut ordered = round_scores
        .drain()
        .map(|(k, v)| (k, target.abs_diff(v)))
        .collect::<Vec<(PlayerId, Duration)>>();

    ordered.sort_by(|a, b| a.1.cmp(&b.1));

    for (i, (plid, delta)) in ordered
        .into_iter()
        .take(scores_by_position.len())
        .enumerate()
    {
        let points = scores_by_position[i];
        let _ = cx.leaderboard.add_score(plid, points as i32).await;
        tracing::info!(
            "Player {} scored {} points (delta: {:?})",
            plid,
            points,
            delta
        );
    }

    cx.insim
        .send_message(&format!("Round {} complete!", round), ConnectionId::ALL)
        .await
        .unwrap();

    if round + 1 >= combo.extensions().rounds {
        // TODO: send leaderboard
        Ok(GameState::Victory)
    } else {
        Ok(GameState::Round {
            round: round + 1,
            combo,
        })
    }
}

pub async fn victory(cx: Context) -> anyhow::Result<GameState> {
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

    Ok(GameState::Idle)
}
