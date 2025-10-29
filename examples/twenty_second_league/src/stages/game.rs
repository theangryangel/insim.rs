use std::{collections::HashMap, time::Duration};

use insim::{
    Packet,
    identifiers::{ConnectionId, PlayerId},
};
use kitcar::{leaderboard::Leaderboard, time::countdown::Countdown};
use tokio::time::sleep;

use crate::{
    GameState, MyState,
    components::{RootPhase, RootProps},
};

pub async fn game(
    insim: insim::builder::SpawnedHandle,
    state: MyState,
) -> anyhow::Result<GameState> {
    let mut packets = insim.subscribe();

    let target = Duration::from_secs(20); // FIXME: from config
    let rounds = 5; // FIXME: pull from config
    let mut round_scores: HashMap<PlayerId, Duration> = HashMap::new();
    let max = 10; // FIXME: pull from config. only max players get score.
    let mut leaderboard = Leaderboard::new();

    for round in 1..=rounds {
        insim.send_command(&format!("/restart")).await.unwrap();

        let _ = state.ui.update(RootProps {
            show: true,
            phase: RootPhase::Restarting,
        });

        println!("Starting round {}/{}", round, rounds);

        round_scores.clear();

        println!("Waiting for game to start");

        // TODO: how do we prevent this from causing issues
        // its convenient. do we care?
        state.game.wait_for_racing().await;
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

                        let _ = state.ui.update(RootProps {
                            show: true,
                            phase: RootPhase::Game {
                                round: round,
                                total_rounds: rounds,
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

        for (i, (plid, delta)) in ordered.into_iter().take(max).enumerate() {
            let points = max - i;
            let _ = leaderboard.add_score(plid, points as i32);
            println!(
                "Player {} scored {} points (delta: {:?})",
                plid, points, delta
            );
        }

        insim
            .send_message(&format!("Round {} complete!", round), ConnectionId::ALL)
            .await
            .unwrap();

        // self.show_leaderboard(false).await?;
    }

    // TODO: show victor

    let _ = state.ui.update(RootProps {
        show: true,
        phase: RootPhase::Victory,
    });

    Ok(GameState::Idle)
}
