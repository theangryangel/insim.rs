//! Twenty Second League
use std::{collections::HashMap, fs, time::Duration};

use eyre::Context as _;
use insim::{
    core::vehicle::Vehicle,
    identifiers::ConnectionId,
    insim::{Mal, Mso, Mst, Mtc, PlcAllowedCarsSet, Res, SmallType},
};
use kitcar::{Context, Engine, Timer, Workshop};
use rand::seq::IndexedRandom;

mod config;
pub mod dictator;

use config::Config;

use crate::config::Combo;

/// Represents the state of the mini-game.
#[derive(Debug)]
pub enum GameState {
    /// Waiting for players to take over server state
    Idle,
    /// 5-minute countdown before the game starts.
    Lobby {
        /// Countdown to game start
        countdown: Timer,
    },
    /// Main game loop is in progress.
    InGame {
        /// Current round
        round: u8,
        /// Time until next round
        timer: Timer,
    },
}

/// State
#[derive(Debug)]
pub struct State {
    state: GameState,

    available_combos: Vec<Combo>,

    combo: Option<Combo>,
}

/// Manages the initial setup of the game, including loading the track and layout
#[derive(Debug)]
pub struct Control;

impl Engine<State> for Control {
    fn mso(&mut self, context: &mut Context<State>, mso: &Mso) {
        match mso.msg.as_str() {
            // FIXME: check if admin
            "!lobby" if matches!(context.state.state, GameState::Idle) => {
                if let Some(combo) = context.state.available_combos.choose(&mut rand::rng()) {
                    context.state.state = GameState::Lobby {
                        countdown: Timer::repeating(Duration::from_secs(10), Some(30)),
                    };

                    context.state.combo = Some(combo.clone());

                    // TODO: use partition
                    let mut plc = PlcAllowedCarsSet::default();
                    let mut mal = Mal::default();
                    for vehicle in combo.vehicles.iter() {
                        match vehicle {
                            Vehicle::Mod(_) => {
                                let _ = mal.insert(vehicle.clone());
                            },
                            Vehicle::Unknown => {},
                            o => {
                                let _ = plc.insert(o.clone());
                            },
                        };
                    }

                    context.queue_packet(SmallType::Alc(plc));
                    context.queue_packet(mal);
                } else {
                    context.queue_packet(Mtc {
                        ucid: mso.ucid,
                        text: "No valid combos found. Invalid config.yaml?".to_owned(),
                        ..Default::default()
                    });
                }
            },
            // FIXME: check if admin
            "!end" => {
                context.state.state = GameState::Idle;
                // TODO: clear combo, etc.
            },
            _ => {},
        }
    }
}

/// Manages a 5-minute countdown before the game begins.
#[derive(Debug)]
pub struct CountdownEngine;

impl Engine<State> for CountdownEngine {
    fn tick(&mut self, context: &mut Context<State>) {
        if let GameState::Lobby {
            countdown: ref timer,
        } = context.state.state
        {
            if timer.tick() {
                if timer.is_finished() {
                    context.state.state = GameState::InGame {
                        round: 0,
                        timer: Timer::repeating(Duration::from_secs(60), Some(15)),
                    };
                    context.queue_packet(Mtc {
                        text: format!("The game is starting in 1 minute. Good luck!"),
                        ucid: ConnectionId::ALL,
                        ..Default::default()
                    });

                    context.queue_packet(Mst {
                        msg: "/laps 1".to_owned(),
                        ..Default::default()
                    });
                    context.queue_packet(Mst {
                        msg: "/restart".to_owned(),
                        ..Default::default()
                    });
                } else {
                    let remaining = timer.remaining_duration();
                    let seconds = remaining.as_secs() % 60;
                    let minutes = (remaining.as_secs() / 60) % 60;

                    // TODO: convert this to use buttons
                    context.queue_packet(Mtc {
                        text: format!("Game starts in {:02}:{:02}", minutes, seconds),
                        ucid: ConnectionId::ALL,
                        ..Default::default()
                    });
                }
            }
        }
    }
}

/// Manages the rounds through a repeating timer in the GameState
#[derive(Debug, Default)]
pub struct Rounds {
    /// temporary results uname:duration
    results: HashMap<String, Duration>,
}

impl Engine<State> for Rounds {
    fn active(&self, context: &Context<State>) -> bool {
        matches!(context.state.state, GameState::InGame { .. })
    }

    fn res(&mut self, context: &mut Context<State>, res: &Res) {
        let _ = self.results.insert(res.uname.clone(), res.ttime);

        // 20s exactly!
        if res.ttime == Duration::from_secs(20) {
            context.queue_packet(Mtc {
                text: format!("Congrats {}, you got an exact 20s lap!", res.pname,),
                ucid: ConnectionId::ALL,
                ..Default::default()
            });
        }
    }

    fn tick(&mut self, context: &mut Context<State>) {
        if let GameState::InGame {
            mut round,
            ref mut timer,
            ..
        } = context.state.state
        {
            if timer.tick() {
                if round > 0 {
                    // TODO store results
                }
                self.results.clear();
                round = round + 1;

                if timer.is_finished() {
                    context.state.state = GameState::Idle;

                    context.queue_packet(Mtc {
                        text: "The game has ended! Thanks for playing".to_owned(),
                        ucid: ConnectionId::ALL,
                        ..Default::default()
                    });
                    // TODO: announce winners, etc.
                    return;
                }

                context.queue_packet(Mst {
                    msg: format!("/laps {}", context.state.combo.as_ref().unwrap().laps),
                    ..Default::default()
                });
                context.queue_packet(Mst {
                    msg: "/restart".to_owned(),
                    ..Default::default()
                });
                context.queue_packet(Mtc {
                    text: format!("Round {} is now starting! Good luck!", round),
                    ucid: ConnectionId::ALL,
                    ..Default::default()
                });
            }
        }
    }
}

fn main() -> eyre::Result<()> {
    let config: Config = serde_norway::from_str(
        &fs::read_to_string("config.yaml").wrap_err("could not read config.yaml")?,
    )
    .wrap_err("Could not parse config.yaml")?;

    let state = State {
        state: GameState::Idle,
        available_combos: config.combo.clone(),
        combo: None,
    };

    Workshop::with_state(state)
        .add_engine(Control)
        .add_engine(CountdownEngine)
        .add_engine(Rounds::default())
        .add_engine(dictator::NoVote)
        .ignition(
            insim::tcp(config.addr)
                .isi_iname(config.iname)
                .isi_admin_password(config.admin)
                .isi_prefix('!')
                .set_non_blocking(true),
        )
        .wrap_err("Failed to execute kitcar ignition")?
        .run(Duration::from_millis(1000 / config.tick_rate.unwrap_or(64)));

    Ok(())
}
