use std::{collections::HashMap, time::Duration};

use insim::{
    Packet,
    identifiers::{ConnectionId, PlayerId},
};
use kitcar::{combos::Combo, game::GameHandle, leaderboard::LeaderboardHandle, time::countdown::Countdown};
use tokio::time::sleep;

use crate::{
    combo::ComboExt, components::{RootPhase, RootProps}, GameState, MyUi
};

pub async fn round(
    insim: insim::builder::SpawnedHandle,
    leaderboard: LeaderboardHandle<PlayerId>,
    round: u32,
    combo: Combo<ComboExt>,
    game: GameHandle,
    ui: MyUi,
    scores_by_position: Vec<i32>,
) -> anyhow::Result<GameState> {
    let mut packets = insim.subscribe();

    let target = combo.extensions().target_time;
    let rounds = combo.extensions().rounds;
    let mut round_scores: HashMap<PlayerId, Duration> = HashMap::new();

    insim.send_command("/restart").await?;

    let _ = ui.update(RootProps {
        show: true,
        phase: RootPhase::Restarting,
    });

    println!("Starting round {}/{}", round, rounds);

    println!("Waiting for game to start");

    // TODO: how do we prevent this from causing issues
    // its convenient. do we care?
    game.wait_for_racing().await;
    sleep(Duration::from_secs(11)).await;

    insim
        .send_message(
            &format!("Round {}/{} - Get close to 20s!", round, rounds),
            ConnectionId::ALL,
        )
        .await
        .unwrap();

    let mut countdown = Countdown::new(
        Duration::from_secs(1),
        60, // FIXME: pull from config
    );

    loop {
        tokio::select! {
            remaining = countdown.tick() => match remaining {
                Some(_) => {
                    let remaining_duration = countdown.remaining_duration();
                    println!("{:?}s remaining!", &remaining_duration);

                    let _ = ui.update(RootProps {
                        show: true,
                        phase: RootPhase::Game {
                            round: round  as usize,
                            total_rounds: rounds as usize,
                            remaining: remaining_duration
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
                    insim
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

    // score round
    let mut ordered = round_scores
        .drain()
        .map(|(k, v)| (k, target.abs_diff(v)))
        .collect::<Vec<(PlayerId, Duration)>>();

    ordered.sort_by(|a, b| a.1.cmp(&b.1));

    for (i, (plid, delta)) in ordered.into_iter().take(scores_by_position.len()).enumerate() {
        let points = scores_by_position[i];
        let _ = leaderboard.add_score(plid, points as i32).await;
        println!(
            "Player {} scored {} points (delta: {:?})",
            plid, points, delta
        );
    }

    insim
        .send_message(&format!("Round {} complete!", round), ConnectionId::ALL)
        .await
        .unwrap();

    if round + 1 >= combo.extensions().rounds {
        // TODO: send leaderboard
        Ok(GameState::Victory)
    } else {
        Ok(GameState::Round { round: round + 1, combo })
    }
}


pub async fn victory(
    _insim: insim::builder::SpawnedHandle, _leaderboard: LeaderboardHandle<PlayerId>,
) -> anyhow::Result<GameState> {
    Ok(GameState::Idle)
}

