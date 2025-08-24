//! Twenty Second League
use std::{collections::HashMap, fs, time::Duration};

use insim::{
    identifiers::ConnectionId,
    insim::{Mso, Mst, Mtc, TinyType, Vtn},
    Packet,
};
use kitcar::{Context, Engine, Timer, Workshop};

#[derive(Debug, serde::Deserialize)]
/// Combo
pub struct Combo {
    /// Name
    pub name: String,
    /// Track to load
    pub track: String,
    /// Track layout
    pub layout: String,
    /// Lap count
    pub laps: u8,
    /// Valid vehicles
    // FIXME: should be a Vehicle, not a String
    pub vehicles: Vec<String>,
}

/// Config
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// Insim IName
    pub iname: String,
    /// Server address
    pub addr: String,
    /// admin password
    pub admin: String,
    /// Combination
    pub combo: Vec<Combo>,
}

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
    InProgress {
        /// Current round
        round: u8,
        /// Time until next round
        timer: Timer,
    },
}

/// Manages the initial setup of the game, including loading the track and layout
#[derive(Debug)]
pub struct Control;

impl Engine<GameState> for Control {
    fn packet(&mut self, context: &mut Context<GameState>, packet: &insim::Packet) {
        if let Packet::Mso(Mso { msg, .. }) = packet {
            match msg.as_ref() {
                // FIXME: check if admin
                "!start" if matches!(context.state, GameState::Idle) => {
                    // FIXME we need to select a random combo from the config.toml
                    // and then load it
                    context.state = GameState::Lobby {
                        countdown: Timer::repeating(Duration::from_secs(10), Some(30)),
                    };
                },
                // FIXME: check if admin
                "!abort" => {
                    context.state = GameState::Idle;
                },
                _ => {},
            }
        }
    }
}

/// Manages a 5-minute countdown before the game begins.
#[derive(Debug)]
pub struct CountdownEngine;

impl Engine<GameState> for CountdownEngine {
    fn tick(&mut self, context: &mut Context<GameState>) {
        if let GameState::Lobby {
            countdown: ref timer,
        } = context.state
        {
            if timer.tick() {
                if timer.is_finished() {
                    context.state = GameState::InProgress {
                        round: 0,
                        timer: Timer::repeating(Duration::from_secs(60), Some(15)),
                    };
                    context.queue_packet(Mtc {
                        text: format!("The game is starting in 1 minute. Good luck!"),
                        ucid: ConnectionId::ALL,
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

impl Engine<GameState> for Rounds {
    fn packet(&mut self, context: &mut Context<GameState>, packet: &insim::Packet) {
        if !matches!(context.state, GameState::InProgress { .. }) {
            return;
        }

        if let Packet::Res(res) = packet {
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
    }

    fn tick(&mut self, context: &mut Context<GameState>) {
        if let GameState::InProgress {
            mut round,
            ref mut timer,
        } = context.state
        {
            if timer.tick() {
                if round > 0 {
                    // TODO store results
                }
                self.results.clear();
                round = round + 1;

                if timer.is_finished() {
                    context.state = GameState::Idle;

                    context.queue_packet(Mtc {
                        text: "The game has ended! Thanks for playing".to_owned(),
                        ucid: ConnectionId::ALL,
                        ..Default::default()
                    });
                    // TODO: announce winners, etc.
                    return;
                }

                context.queue_packet(Mst {
                    msg: "/laps 1".to_owned(),
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

/// Prevent voting when a game is in progress
#[derive(Debug)]
pub struct Dictator;

impl Engine<GameState> for Dictator {
    fn packet(&mut self, context: &mut Context<GameState>, packet: &insim::Packet) {
        if !matches!(context.state, GameState::InProgress { .. }) {
            return;
        }

        if let Packet::Vtn(Vtn { .. }) = packet {
            context.queue_packet(TinyType::Vtc);
        }
    }
}

fn main() {
    // FIXME - unwrap
    let config: Config = toml::from_str(&fs::read_to_string("config.toml").unwrap()).unwrap();

    Workshop::with_state(GameState::Idle)
        .add_engine(Control)
        .add_engine(CountdownEngine)
        .add_engine(Rounds::default())
        .add_engine(Dictator)
        .ignition(
            insim::tcp(config.addr.clone())
                .isi_iname(config.iname.clone())
                .isi_prefix('!')
                .set_non_blocking(true),
        )
        .run(Duration::from_millis(1000 / 60));
}
