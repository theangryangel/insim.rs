use std::{collections::HashMap, time::Duration};

use insim::{
    Packet,
    identifiers::{ConnectionId, PlayerId},
};
use kitcar::{
    leaderboard::{Leaderboard, LeaderboardHandle},
    presence::PresenceHandle,
    time::countdown::Countdown,
    ui,
};
use tokio::{sync::broadcast, time::sleep};

use crate::{
    components::{Root, RootPhase, RootProps},
    phases::Transition,
};

pub(crate) struct PhaseGame {
    insim: insim::builder::SpawnedHandle,
    presence: PresenceHandle,
    leaderboard: Leaderboard<PlayerId>,
    ui: ui::ManagerHandle<Root>,
    rounds: usize,
    round_scores: HashMap<PlayerId, Duration>,
    target: Duration,
}

impl PhaseGame {
    pub(crate) fn spawn(
        insim: insim::builder::SpawnedHandle,
        presence: PresenceHandle,
        ui: ui::ManagerHandle<Root>,
    ) -> tokio::task::JoinHandle<Transition> {
        let inst = Self {
            insim,
            presence,
            ui,
            rounds: 5,
            round_scores: HashMap::new(),
            target: Duration::from_secs(20),
            leaderboard: Leaderboard::new(),
        };
        tokio::spawn(inst.run())
    }

    pub(crate) async fn run(mut self) -> Transition {
        let mut packets = self.insim.subscribe();

        for round in 1..=self.rounds {
            self.insim.send_command(&format!("/restart")).await.unwrap();

            let _ = self.ui.update(RootProps {
                show: true,
                phase: RootPhase::Restarting,
            });

            println!("Starting round {}/{}", round, self.rounds);

            self.round_scores.clear();

            // FIXME: add a wait_for_restart to game state?
            loop {
                match packets.recv().await {
                    Ok(Packet::Rst(_)) => {
                        // TODO: how do we prevent this from causing issues
                        // its convenient. do we care?
                        sleep(Duration::from_secs(11)).await;
                        break;
                    },
                    p => {
                        println!("{:?}", p);
                    },
                }
            }

            self.run_round(round, &mut packets).await.unwrap();
            self.score_round(10).await;

            self.insim
                .send_message(&format!("Round {} complete!", round), ConnectionId::ALL)
                .await
                .unwrap();

            let _ = self.ui.update(RootProps {
                show: true,
                phase: RootPhase::Victory,
            });
            // self.show_leaderboard(false).await?;
        }

        Transition::Idle
    }

    async fn run_round(
        &mut self,
        round: usize,
        game_rx: &mut broadcast::Receiver<Packet>,
    ) -> anyhow::Result<()> {
        self.insim
            .send_message(
                &format!("Round {}/{} - Get close to 20s!", round, self.rounds),
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

                        let _ = self.ui.update(RootProps {
                            show: true,
                            phase: RootPhase::Game {
                                round: round,
                                total_rounds: self.rounds,
                                remaining: remaining_duration
                            }
                        });
                    },
                    None => {
                        break;
                    }
                },
                packet = game_rx.recv() => match packet {
                    Ok(packet) => { println!("{:?}", packet); self.handle_packet(&packet).await?; },
                    _ => {}
                }
            }
        }

        Ok(())
    }

    async fn handle_packet(&mut self, packet: &Packet) -> anyhow::Result<()> {
        match packet {
            Packet::Fin(fin) => {
                let _ = self.round_scores.insert(fin.plid, fin.ttime);
            },
            Packet::Ncn(ncn) => {
                self.insim
                    .send_message(
                        "Welcome to 20 Second League! Get as close to 20s as possible.",
                        ncn.ucid,
                    )
                    .await
                    .unwrap();
            },
            Packet::Pll(pll) => {
                // FIXME: probably unfair, but fuck it for now
                let _ = self.round_scores.remove(&pll.plid);
            },
            _ => {},
        }

        Ok(())
    }

    async fn score_round(&mut self, max: usize) {
        let mut ordered = self
            .round_scores
            .drain()
            .map(|(k, v)| (k, self.target.abs_diff(v)))
            .collect::<Vec<(PlayerId, Duration)>>();

        ordered.sort_by(|a, b| a.1.cmp(&b.1));

        for (i, (plid, delta)) in ordered.into_iter().take(max).enumerate() {
            let points = max - i;
            let _ = self.leaderboard.add_score(plid, points as i32);
            println!(
                "Player {} scored {} points (delta: {:?})",
                plid, points, delta
            );
        }
    }
}
