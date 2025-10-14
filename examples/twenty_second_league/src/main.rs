//! 20s league
mod combo;
mod components;

use std::{collections::HashMap, fs, time::Duration};

use anyhow::{Context, Result};
use insim::{
    identifiers::{ConnectionId, PlayerId},
    insim::TinyType,
    Packet, WithRequestId,
};
use kitcar::{
    leaderboard::{Leaderboard, LeaderboardHandle},
    presence::{Presence, PresenceHandle},
    time::countdown::Countdown,
    ui::UiManager,
    utils::NoVote,
    Service, State as _,
};
use tokio::{
    sync::{broadcast, watch},
    time::sleep,
};

const ROUNDS_PER_GAME: usize = 5;
const ROUND_DURATION: u32 = 30;
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
    let presence = Presence::spawn(insim.clone());
    let leaderboard = Leaderboard::spawn(insim.clone());
    NoVote::spawn(insim.clone());

    let (mut game, signals_rx) =
        TwentySecondLeague::new(config, leaderboard, presence, insim.clone());
    let _ = UiManager::spawn::<components::Root>(signals_rx, insim.clone());

    let _ = insim.send(TinyType::Ncn.with_request_id(1)).await;
    let _ = insim.send(TinyType::Npl.with_request_id(2)).await;

    println!("20 Second League started!");

    loop {
        game.wait_for_players().await?;
        // TODO: select a combo
        game.run().await?;
        game.show_leaderboard(true).await?;

        sleep(Duration::from_secs(30)).await;
        game.reset().await?;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Phase {
    Idle,
    Game { round: usize, remaining: Duration },
    Victory,
}

struct TwentySecondLeague {
    config: Config,

    round_scores: HashMap<PlayerId, Duration>,
    insim: insim::builder::SpawnedHandle,

    leaderboard: LeaderboardHandle,
    presence: PresenceHandle,

    // TODO: hack
    signals_tx: watch::Sender<Phase>,
}

impl TwentySecondLeague {
    fn new(
        config: Config,
        leaderboard: LeaderboardHandle,
        presence: PresenceHandle,
        insim: insim::builder::SpawnedHandle,
    ) -> (Self, watch::Receiver<Phase>) {
        let (signals_tx, signals_rx) = watch::channel(Phase::Idle);

        (
            Self {
                config,
                round_scores: HashMap::new(),
                insim,
                leaderboard,
                presence,
                signals_tx,
            },
            signals_rx,
        )
    }

    async fn reset(&mut self) -> Result<()> {
        self.leaderboard.clear().await;
        self.round_scores.clear();
        Ok(())
    }

    async fn wait_for_players(&self) -> Result<()> {
        loop {
            let count = self.presence.player_count().await;

            if count >= 1 {
                self.message_all("Starting in 10s").await;
                sleep(Duration::from_secs(2)).await;
                return Ok(());
            }

            self.message_all("Waiting for players...").await;
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn run(&mut self) -> Result<()> {
        for round in 1..=self.config.rounds.unwrap_or(5) {
            let mut game_rx = self.insim.subscribe();

            self.command(&format!("/restart")).await;

            let _ = self.signals_tx.send(Phase::Game {
                round: round,
                remaining: Duration::from_secs(60), // FIXME: pull from config
            });

            println!("Starting round {}/{}", round, ROUNDS_PER_GAME);

            self.round_scores.clear();

            loop {
                match game_rx.recv().await {
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

            self.run_round(&round, &mut game_rx).await?;
            self.score_round(10).await;

            self.message_all(&format!("Round {} complete!", round))
                .await;

            let _ = self.signals_tx.send(Phase::Victory);

            self.show_leaderboard(false).await?;
        }

        Ok(())
    }

    async fn run_round(
        &mut self,
        round: &usize,
        game_rx: &mut broadcast::Receiver<Packet>,
    ) -> Result<()> {
        self.message_all(&format!(
            "Round {}/{} - Get close to 20s!",
            round, ROUNDS_PER_GAME
        ))
        .await;

        let mut countdown = Countdown::new(
            Duration::from_secs(1),
            60, // FIXME: pull from config
        );

        loop {
            tokio::select! {
                remaining = countdown.tick() => match remaining {
                    Some(_) => {
                        let remaining_duration = countdown.remaining_duration().await;
                        self.message_all(&format!("{:?}s remaining!", &remaining_duration)).await;

                        let _ = self.signals_tx.send(Phase::Game {
                            round: *round,
                            remaining: remaining_duration
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

    async fn handle_packet(&mut self, packet: &Packet) -> Result<()> {
        match packet {
            Packet::Fin(fin) => {
                let _ = self.round_scores.insert(fin.plid, fin.ttime);
            },
            Packet::Ncn(ncn) => {
                self.message(
                    &ncn.ucid,
                    "Welcome to 20 Second League! Get as close to 20s as possible.",
                )
                .await;
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
            .map(|(k, v)| (k, Duration::from_secs(TARGET_TIME as u64).abs_diff(v)))
            .collect::<Vec<(PlayerId, Duration)>>();

        ordered.sort_by(|a, b| a.1.cmp(&b.1));

        for (i, (plid, delta)) in ordered.into_iter().take(max).enumerate() {
            let points = max - i;
            let _ = self
                .leaderboard
                .add_player_score(&plid, points as i32)
                .await;
            println!(
                "Player {} scored {} points (delta: {:?})",
                plid, points, delta
            );
        }
    }

    async fn show_leaderboard(&self, finished: bool) -> Result<()> {
        let rankings = self.leaderboard.ranking(Some(10)).await;

        self.message_all("=== Leaderboard ===").await;
        for (i, (plid, score)) in rankings.iter().enumerate() {
            // TODO: shoudl really collect the playerinfo up front, but whatever
            if let Some(playerinfo) = self.presence.player(plid).await {
                self.message_all(&format!(
                    "{}. Player {} - {} pts\n",
                    i + 1,
                    playerinfo.pname,
                    score
                ))
                .await;
            }
        }

        if finished {
            if let Some((winner_plid, winner_score)) = rankings.first() {
                self.message_all(&format!(
                    "Winner: Player {} with {} points!",
                    winner_plid, winner_score
                ))
                .await;
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
