//! 20s league
mod combo;
mod components;
mod no_vote;

use std::{collections::HashMap, fs, time::Duration};

use anyhow::{Context, Result};
use insim::{Packet, WithRequestId, identifiers::ConnectionId, insim::TinyType};
use kitcar::{
    Service, leaderboard::Leaderboard, presence::Presence, time::countdown::Countdown, ui,
};
use tokio::{sync::watch, time::sleep};

use crate::components::{RootPhase, RootProps};

const ROUNDS_PER_GAME: usize = 5;
const TARGET_TIME: f32 = 20.0;

/// Config
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// Insim IName
    pub iname: Option<String>,
    /// Server address
    pub addr: String,
    /// admin password
    pub admin: Option<String>,
    /// Warmup duration
    #[serde(with = "humantime_serde")]
    pub warmup_duration: Duration,
    /// Combinations
    pub combos: combo::ComboCollection,
    /// Number of rounds
    pub rounds: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config: Config = serde_norway::from_str(
        &fs::read_to_string("config.yaml").context("could not read config.yaml")?,
    )
    .context("Could not parse config.yaml")?;

    let (insim, _join_handle) = insim::tcp(config.addr.as_str())
        .isi_admin_password(config.admin.clone())
        .isi_iname(config.iname.clone())
        .spawn(100)
        .await?;

    // Spawn library handles
    no_vote::NoVote::spawn(insim.clone());

    let (mut game, signals_rx) = TwentySecondLeague::new(config, insim.clone());
    let _ = ui::Manager::spawn::<components::Root>(signals_rx, insim.clone());

    let _ = insim.send(TinyType::Ncn.with_request_id(1)).await;
    let _ = insim.send(TinyType::Npl.with_request_id(2)).await;

    let mut packet_rx = insim.subscribe();

    println!("20 Second League started!");

    loop {
        tokio::select! {
            packet = packet_rx.recv() => {
                let _ = game.handle_packet(&packet?).await;
            },
            res = game.run_phase() => {
                // FIXME? or is this ok? seems fine probably.
                game.phase = res?;
            },
        }
    }
}

#[derive(Debug)]
enum Phase {
    Idle,
    Game {
        round_scores: HashMap<String, Duration>, // by uname
        leaderboard: Leaderboard<String>,        // by uname
    },
    Victory,
}

struct TwentySecondLeague {
    phase: Phase,
    config: Config,
    insim: insim::builder::SpawnedHandle,
    presence: Presence,

    signals_tx: watch::Sender<RootProps>,
}

impl TwentySecondLeague {
    fn new(
        config: Config,
        insim: insim::builder::SpawnedHandle,
    ) -> (Self, watch::Receiver<RootProps>) {
        let (signals_tx, signals_rx) = watch::channel(RootProps {
            phase: RootPhase::Idle,
            show: true,
        });

        (
            Self {
                phase: Phase::Idle,
                config,
                insim,
                presence: Presence::new(),
                signals_tx,
            },
            signals_rx,
        )
    }

    async fn run_phase(&mut self) -> Result<Phase> {
        match self.phase {
            Phase::Idle => {
                self.wait_for_players().await?;
                Ok(Phase::Game {
                    leaderboard: Leaderboard::new(),
                    round_scores: HashMap::new(),
                })
            },
            Phase::Game { .. } => {
                self.run_game().await?;
                self.show_leaderboard(true).await?;
                // TODO: we need to pass over the victory player info
                Ok(Phase::Victory)
            },
            Phase::Victory => {
                sleep(Duration::from_secs(30)).await;
                Ok(Phase::Idle)
            },
        }
    }

    async fn wait_for_players(&self) -> Result<()> {
        loop {
            let count = self.presence.player_count();

            if count >= 1 {
                self.message_all("Starting in 10s").await;
                sleep(Duration::from_secs(2)).await;
                return Ok(());
            }

            self.message_all("Waiting for players...").await;
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn run_game(&mut self) -> Result<()> {
        for round in 1..=self.config.rounds.unwrap_or(5) {
            let mut game_rx = self.insim.subscribe();

            self.command(&format!("/restart")).await;

            let _ = self.signals_tx.send(RootProps {
                show: true,
                phase: RootPhase::Game {
                    round: round,
                    remaining: Duration::from_secs(60), // FIXME: pull from config
                },
            });

            println!("Starting round {}/{}", round, ROUNDS_PER_GAME);

            if let Phase::Game {
                ref mut round_scores,
                ..
            } = self.phase
            {
                round_scores.clear();
            }

            // FIXME: turn this into wait_for_race_start
            loop {
                match game_rx.recv().await {
                    Ok(Packet::Rst(_)) => {
                        sleep(Duration::from_secs(11)).await;
                        break;
                    },
                    p => {
                        println!("{:?}", p);
                    },
                }
            }

            self.run_round(&round).await?;
            self.score_round(10).await;

            self.message_all(&format!("Round {} complete!", round))
                .await;

            let _ = self.signals_tx.send(RootProps {
                show: true,
                phase: RootPhase::Victory,
            });

            self.show_leaderboard(false).await?;
        }

        Ok(())
    }

    async fn run_round(&mut self, round: &usize) -> Result<()> {
        self.message_all(&format!(
            "Round {}/{} - Get close to 20s!",
            round, ROUNDS_PER_GAME
        ))
        .await;

        let mut countdown = Countdown::new(
            Duration::from_secs(1),
            60, // FIXME: pull from config
        );

        while let Some(_remaining) = countdown.tick().await {
            let remaining_duration = countdown.remaining_duration();
            self.message_all(&format!("{:?}s remaining!", &remaining_duration))
                .await;

            let _ = self.signals_tx.send(RootProps {
                show: true,
                phase: RootPhase::Game {
                    round: *round,
                    remaining: remaining_duration,
                },
            });
        }

        Ok(())
    }

    async fn handle_packet(&mut self, packet: &Packet) -> Result<()> {
        self.presence.handle_packet(packet);

        match packet {
            Packet::Fin(fin) => {
                if_chain::if_chain! {
                    if let Phase::Game { ref mut round_scores, .. } = self.phase;
                    if let Some(player_info) = self.presence.player(&fin.plid);
                    if !player_info.ptype.is_ai();
                    if let Some(connection_info) = self.presence.connection(&player_info.ucid);
                    then {
                        let _ = round_scores.insert(connection_info.uname.clone(),fin.ttime);
                    }
                }
            },
            Packet::Ncn(ncn) => {
                self.message(
                    &ncn.ucid,
                    "Welcome to 20 Second League! Get as close to 20s as possible.",
                )
                .await;
            },
            _ => {},
        }

        Ok(())
    }

    async fn score_round(&mut self, max: usize) {
        if let Phase::Game {
            ref mut round_scores,
            ref mut leaderboard,
        } = self.phase
        {
            let mut ordered = round_scores
                .drain()
                .map(|(k, v)| (k, Duration::from_secs(TARGET_TIME as u64).abs_diff(v)))
                .collect::<Vec<(String, Duration)>>();

            ordered.sort_by(|a, b| a.1.cmp(&b.1));

            for (i, (plid, delta)) in ordered.into_iter().take(max).enumerate() {
                let points = max - i;
                let _ = leaderboard.add_score(plid.clone(), points as i32);
                println!(
                    "Player {} scored {} points (delta: {:?})",
                    plid, points, delta
                );
            }
        }
    }

    async fn show_leaderboard(&self, finished: bool) -> Result<()> {
        if let Phase::Game {
            ref leaderboard, ..
        } = self.phase
        {
            let rankings = leaderboard.top_n_ranking(Some(10));

            self.message_all("=== Leaderboard ===").await;
            for (i, (uname, score)) in rankings.iter().enumerate() {
                // TODO: shoudl really collect the playerinfo up front, but whatever
                self.message_all(&format!("{}. Player {} - {} pts\n", i + 1, uname, score))
                    .await;
            }

            if finished {
                if let Some((winner_uname, winner_score)) = rankings.first() {
                    self.message_all(&format!(
                        "Winner: Player {} with {} points!",
                        winner_uname, winner_score
                    ))
                    .await;
                }
            }
        }

        Ok(())
    }

    async fn message_all(&self, msg: &str) {
        println!("{}", msg);
        let _ = self.insim.send_message(msg, ConnectionId::ALL).await;
    }

    async fn message(&self, ucid: &ConnectionId, msg: &str) {
        println!("{}", msg);
        let _ = self.insim.send_message(msg, *ucid).await;
    }

    async fn command(&self, command: &str) {
        println!("{}", command);

        let _ = self.insim.send_command(command).await;
    }
}
