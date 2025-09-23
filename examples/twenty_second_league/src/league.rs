//! Twenty Second League implementation
use std::{collections::HashMap, fmt::Debug, time::Duration};

use insim::{
    identifiers::ConnectionId,
    insim::{Mso, Mtc},
    Packet,
};
use kitcar::{time::countdown::Countdown, ui::Ui, Plugin, PluginContext};
use tokio::time::sleep;
use tracing::{info, warn};

use crate::Config;

/// League Mini-Game State
#[derive(Debug, Default)]
pub(crate) enum LeagueState {
    #[default]
    Idle,
    Warmup,
    Game {
        round: u32,
    },
}

impl LeagueState {
    async fn tick(&mut self) {
        // TODO: what do we want to output here?
        // is this wise? do we just go for a global tick instead and suck it up?
    }
}

/// League Mini-Game
#[derive(Debug, Default)]
pub(crate) struct League {
    state: LeagueState,
    // TODO: Move into LeagueState
    scoreboard: HashMap<ConnectionId, u32>,
    // TODO: Move into LeagueState and add a tick method?
    timer: Option<Countdown>,

    views: HashMap<ConnectionId, Ui<super::components::countdown>>,
}

impl League {
    async fn tick(&mut self) -> Option<u32> {
        if let Some(timer) = &mut self.timer {
            timer.tick().await
        } else {
            None
        }
    }

    async fn handle_packet(&mut self, packet: Packet, ctx: &PluginContext<Config>) {
        match packet {
            Packet::Ncn(ncn) => {
                let _ = self.scoreboard.insert(ncn.ucid, 0);
            },

            Packet::Cnl(cnl) => {
                let _ = self.scoreboard.remove(&cnl.ucid);
            },

            Packet::Mso(mso) => {
                self.handle_chat(mso, ctx).await;
            },

            Packet::Res(res) => {
                // TODO: scoring based on round end

                if_chain::if_chain! {
                    if matches!(self.state, LeagueState::Game{ .. });
                    if let Some(info) = ctx.get_player(res.plid).await;
                    if !info.ptype.is_ai();
                    if res.ttime == Duration::from_secs(20);
                    then {
                        ctx.send_packet(Mtc {
                            text: format!("Congrats {}, you got an exact 20s lap!", res.pname),
                            ucid: ConnectionId::ALL,
                            ..Default::default()
                        }).await;
                    }
                }
            },
            _ => {},
        }
    }

    async fn handle_chat(&mut self, mso: Mso, ctx: &PluginContext<Config>) {
        // FIXME unwrap
        let is_admin = if let Some(info) = ctx.get_connection(mso.ucid).await {
            info.admin
        } else {
            false
        };

        // TODO: command router thing? clap but for chat commands would be cool.

        match mso.msg_from_textstart() {
            "!start" => {
                if matches!(self.state, LeagueState::Idle) && is_admin {
                    info!("Starting 20 Second League warmup");
                    ctx.send_message("^3Starting 20 Second League! Warmup phase beginning...")
                        .await;

                    self.select_and_set_track(&ctx).await;
                    self.state = LeagueState::Warmup;
                    self.timer = Some(Countdown::new(Duration::from_secs(1), 300));
                }
            },

            "!stop" => {
                if !matches!(self.state, LeagueState::Idle) && is_admin {
                    ctx.send_message("^320 Second League aborted!").await;
                    self.state = LeagueState::Idle;
                }
            },

            "!status" => {
                let status = match self.state {
                    LeagueState::Idle => "Idle",
                    LeagueState::Warmup { .. } => "Warm up",
                    LeagueState::Game { .. } => "In-Game",
                };
                ctx.send_packet(Mtc {
                    ucid: mso.ucid,
                    text: status.into(),
                    ..Default::default()
                })
                .await;
            },

            _ => {},
        }
    }

    async fn select_and_set_track(&mut self, ctx: &PluginContext<Config>) {
        let combos = &ctx.user_state.combo;
        if combos.is_empty() {
            warn!("No tracks configured for 20 Second League");
            return;
        }

        let combo = combos.random().unwrap(); // FIXME

        ctx.send_command("/end").await;
        sleep(Duration::from_secs(6)).await;
        ctx.send_command("/clear").await;
        ctx.send_command(&format!("/track={}", combo.track)).await;
        ctx.send_command(&format!("/laps={}", combo.laps.unwrap_or(1)))
            .await;
        ctx.send_message(&format!("^3Loading ^6{}. Starting warmup.", combo.name))
            .await;
    }

    async fn start_game(&mut self, ctx: &PluginContext<Config>) {
        info!("Starting 20 Second League game phase");
        ctx.send_message("^2Warmup complete! Starting 20 Second League game phase!")
            .await;
        ctx.send_message("^7Objective: Get as close to 20.000 seconds as possible!")
            .await;
        ctx.send_message("^7Double points for exactly 20.000 seconds!")
            .await;
        self.state = LeagueState::Game { round: 0 };
        self.start_round(ctx).await;
    }

    async fn start_round(&mut self, ctx: &PluginContext<Config>) {
        ctx.send_command("/restart").await;
        if let LeagueState::Game { mut round } = &self.state {
            round = round.saturating_add(1);
            ctx.send_message(&format!(
                "^3Round {}/20 starting! Get as close to 20.000 seconds as possible!",
                round
            ))
            .await;
        }
    }

    async fn end_round(&mut self, ctx: &PluginContext<Config>) {
        ctx.send_command("/restart").await;
        if let LeagueState::Game { mut round } = &self.state {
            round = round.saturating_add(1);
            ctx.send_message(&format!(
                "^3Round {}/20 starting! Get as close to 20.000 seconds as possible!",
                round
            ))
            .await;
        }
    }

    async fn end_game(&mut self, ctx: &PluginContext<Config>) {
        info!("20 Second League game completed");

        ctx.send_message(&format!(
            "^3Thanks for playing! Final results being tallied!"
        ))
        .await;

        // TODO: calculate final standings

        self.state = LeagueState::Idle;
        self.scoreboard.clear();
        self.timer = None;
    }
}

#[async_trait::async_trait]
impl Plugin<Config> for League {
    async fn run(mut self: Box<Self>, ctx: PluginContext<Config>) -> Result<(), ()> {
        let mut packets = ctx.subscribe_to_packets();

        loop {
            tokio::select! {
                // TODO: See LeagueState above for noodling.
                Some(remaining) = self.tick() => match self.state {
                    LeagueState::Idle => unreachable!(),
                    LeagueState::Warmup => {
                        if remaining == 0 {
                            // TODO: check if no players and abort
                            self.start_game(&ctx).await;
                        }
                    },
                    LeagueState::Game { round } => {
                        self.end_round(&ctx).await;
                        if round > ctx.user_state.total_rounds {
                            self.end_game(&ctx).await;
                        } else {
                            self.start_round(&ctx).await;
                        }
                    }
                },

                packet = packets.recv() => {
                    // FIXME: remove unwrap
                    self.handle_packet(packet.unwrap(), &ctx).await;
                },

                _ = ctx.abort() => {
                    break;
                }
            }
        }

        Ok(())
    }
}
