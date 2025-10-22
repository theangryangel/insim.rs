//! 20s league
mod combo;
mod components;
mod no_vote;

use std::{collections::HashMap, fs, time::Duration};

use anyhow::{Context, Result};
use insim::{Packet, WithRequestId, identifiers::ConnectionId, insim::TinyType};
use kitcar::{
    combos::ComboList,
    leaderboard::Leaderboard,
    presence::{Presence, PresenceHandle},
    ui,
};
use tokio::time::{Instant, interval};

use crate::components::{Root, RootPhase, RootProps};

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
    pub combos: ComboList<combo::ComboExt>,
    /// Number of rounds
    pub rounds: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup with a default log level of INFO RUST_LOG is unset
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let config: Config = serde_norway::from_str(
        &fs::read_to_string("config.yaml").context("could not read config.yaml")?,
    )
    .context("Could not parse config.yaml")?;

    let (insim, _join_handle) = insim::tcp(config.addr.as_str())
        .isi_admin_password(config.admin.clone())
        .isi_iname(config.iname.clone())
        .spawn(100)
        .await?;

    let (ui_handle, _ui_thread) = ui::Manager::spawn::<components::Root>(
        insim.clone(),
        RootProps {
            phase: RootPhase::Idle,
            show: true,
        },
    );

    let presence_handle = Presence::spawn(insim.clone(), 32);

    let mut game = TwentySecondLeague::new(config, presence_handle, ui_handle, insim.clone());

    let _ = insim.send(TinyType::Ncn.with_request_id(1)).await;
    let _ = insim.send(TinyType::Npl.with_request_id(2)).await;

    let mut packet_rx = insim.subscribe();

    println!("20 Second League started!");

    let mut tick_interval = interval(Duration::from_millis((1000.0 / 60.0) as u64));

    loop {
        tokio::select! {
            packet = packet_rx.recv() => {
                let _ = game.handle_packet(&packet?).await;
            },
            _ = tick_interval.tick() => {
                game.tick().await?;
            },
        }
    }
}

#[derive(Debug)]
enum Phase {
    Game {
        total_rounds: usize,
        round: usize,
        round_start: Instant,
        round_scores: HashMap<String, Duration>, // by uname
        leaderboard: Leaderboard<String>,        // by uname
        last_prop_update: Instant,
    },
    Victory {
        victory_start: Instant,
    },
}

struct TwentySecondLeague {
    phase: Option<Phase>,
    config: Config,
    insim: insim::builder::SpawnedHandle,
    presence: PresenceHandle,
    ui: ui::ManagerHandle<Root>,
}

impl TwentySecondLeague {
    fn new(
        config: Config,
        presence: PresenceHandle,
        ui: ui::ManagerHandle<Root>,
        insim: insim::builder::SpawnedHandle,
    ) -> Self {
        Self {
            phase: None,
            config,
            insim,
            presence,
            ui,
        }
    }

    /// Tick and then return the next phase
    async fn tick(&mut self) -> Result<()> {
        let phase = self.phase.take();

        let next_phase = match phase {
            None => {
                // Nothing - booting up
                self.phase = None;
                return Ok(());
            },
            Some(Phase::Game {
                total_rounds,
                round,
                round_start,
                mut leaderboard,
                mut round_scores,
                last_prop_update,
            }) => {
                let now = Instant::now();

                let elapsed = now.saturating_duration_since(round_start);
                let round_duration = Duration::from_secs(10); // FIXME: from config
                let remaining = round_duration.saturating_sub(elapsed);

                // Update countdown every second
                let last_prop_update = if last_prop_update.elapsed() > Duration::from_secs(1) {
                    self.ui
                        .update(RootProps {
                            phase: RootPhase::Game {
                                round: round as u32,
                                total_rounds: total_rounds as u32,
                                remaining: remaining,
                            },
                            show: true,
                        })
                        .await;
                    Instant::now()
                } else {
                    last_prop_update
                };

                // Round complete?
                if remaining.is_zero() {
                    // Score the round
                    let max_scoring_players = 10;

                    let mut ordered = round_scores
                        .drain()
                        .map(|(k, v)| (k, Duration::from_secs(TARGET_TIME as u64).abs_diff(v)))
                        .collect::<Vec<(String, Duration)>>();

                    ordered.sort_by(|a, b| a.1.cmp(&b.1));

                    for (i, (plid, delta)) in
                        ordered.into_iter().take(max_scoring_players).enumerate()
                    {
                        let points = max_scoring_players - i;
                        let _ = leaderboard.add_score(plid.clone(), points as i32);
                        println!(
                            "Player {} scored {} points (delta: {:?})",
                            plid, points, delta
                        );
                    }

                    if round >= total_rounds {
                        // Game over
                        self.message_all("Game over!").await;

                        self.ui
                            .update(RootProps {
                                phase: RootPhase::Victory,
                                show: true,
                            })
                            .await;

                        self.show_leaderboard(true).await?;

                        Phase::Victory {
                            victory_start: Instant::now(),
                        }
                    } else {
                        // Next round - auto restart
                        self.show_leaderboard(false).await?;

                        self.command("/restart").await;

                        // FIXME: do we even need this? probably not.
                        self.ui
                            .update(RootProps {
                                phase: RootPhase::Game {
                                    round: (round + 1) as u32,
                                    total_rounds: total_rounds as u32,
                                    remaining: remaining, // FIXME
                                },
                                show: true,
                            })
                            .await;

                        Phase::Game {
                            total_rounds,
                            round: round + 1,
                            round_start: Instant::now(), // TODO; add a fudging factor,
                            round_scores,
                            leaderboard,
                            last_prop_update,
                        }
                    }
                } else {
                    Phase::Game {
                        total_rounds,
                        round,
                        round_start,
                        leaderboard,
                        round_scores,
                        last_prop_update,
                    }
                }
            },
            Some(Phase::Victory { victory_start }) => {
                let now = Instant::now();

                let elapsed = now.saturating_duration_since(victory_start);
                if elapsed >= Duration::from_secs(60) {
                    self.phase = None;
                    return Ok(());
                } else {
                    Phase::Victory { victory_start }
                }
            },
        };

        self.phase = Some(next_phase);

        Ok(())
    }

    async fn handle_packet(&mut self, packet: &Packet) -> Result<()> {
        // self.presence.handle_packet(packet);

        match packet {
            Packet::Fin(fin) => {
                if_chain::if_chain! {
                    if let Some(Phase::Game { ref mut round_scores, .. }) = self.phase;
                    if let Some(player_info) = self.presence.player(&fin.plid).await;
                    if !player_info.ptype.is_ai();
                    if let Some(connection_info) = self.presence.connection(&player_info.ucid).await;
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
            Packet::Mso(mso) => {
                println!("mso = {:?}", mso);
                println!("mso = {:?}", mso.msg_from_textstart());
                println!("presence = {:?}", self.presence);

                if_chain::if_chain! {
                    if mso.msg_from_textstart() == "!start";
                    if self.phase.is_none();
                    if let Some(conn_info) = self.presence.connection(&mso.ucid).await;
                    if conn_info.admin;
                    then {
                        self.message_all("Starting game...").await;

                        self.phase = Some(Phase::Game {
                            total_rounds: 5,
                            round: 1,
                            round_start: Instant::now(),
                            round_scores: HashMap::new(),
                            leaderboard: Leaderboard::new(),
                            last_prop_update: Instant::now(),
                        });
                    }
                }
            },
            _ => {},
        }

        Ok(())
    }

    async fn show_leaderboard(&self, finished: bool) -> Result<()> {
        if let Some(Phase::Game {
            ref leaderboard, ..
        }) = self.phase
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
