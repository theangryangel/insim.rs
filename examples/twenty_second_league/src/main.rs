//! Twenty Second League
use std::{collections::HashMap, time::Duration};

use humanize_duration::prelude::DurationExt;
use insim::{
    identifiers::ConnectionId,
    insim::{Mst, Mtc},
    Packet,
};
use kitcar::{Context, Engine, Timer, Workshop};

/// Represents the state of the mini-game.
#[derive(Debug)]
pub enum GameState {
    /// Waiting for players to take over server state
    Waiting,
    /// 5-minute countdown before the game starts.
    Countdown,
    /// Main game loop is in progress.
    InProgress {
        /// Current round
        round: u8,
        /// Time until next round
        timer: Timer,
    },
    /// Game has ended.
    Ended,
}

/// Manages the initial setup of the game, including loading the track and layout
#[derive(Debug)]
pub struct Setup;

impl Engine<GameState> for Setup {
    fn tick(&mut self, context: &mut Context<GameState>) {
        // FIXME: check LFS game state OR we make this manual. we probably need a MC really.
        if matches!(context.state, GameState::Waiting) {
            context.queue_packet(Mst {
                msg: "/load FE1Y".to_owned(),
                ..Default::default()
            });
            context.queue_packet(Mst {
                msg: "/layout FE1X_20s_cup.lyt".to_owned(),
                ..Default::default()
            });
            context.queue_packet(Mst {
                msg: "/laps 0".to_owned(),
                ..Default::default()
            });
            context.state = GameState::Countdown;
        }
    }
}

/// Manages a 5-minute countdown before the game begins.
#[derive(Debug)]
pub struct CountdownEngine {
    timer: Timer,
}

impl CountdownEngine {
    /// New
    pub fn new() -> Self {
        Self {
            timer: Timer::repeating(Duration::from_secs(10), Some(30)),
        }
    }
}

impl Engine<GameState> for CountdownEngine {
    fn tick(&mut self, context: &mut Context<GameState>) {
        if matches!(context.state, GameState::Countdown) {
            return;
        }

        if self.timer.tick() {
            if self.timer.is_finished() {
                context.state = GameState::InProgress {
                    round: 0,
                    timer: Timer::repeating(Duration::from_secs(60), Some(15)),
                };
                // TODO: RCM more appropriate?
                context.queue_packet(Mtc {
                    text: format!("The game is starting. Good luck!"),
                    ucid: ConnectionId::ALL,
                    ..Default::default()
                });
            } else {
                // TODO: convert this to use buttons
                context.queue_packet(Mtc {
                    text: format!(
                        "Game starts in {}",
                        self.timer
                            .remaining_duration()
                            .human(humanize_duration::Truncate::Millis)
                    ),
                    ucid: ConnectionId::ALL,
                    ..Default::default()
                });
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
            let uname = "PLACEHOLDER".to_owned();
            let _ = self.results.insert(uname, res.ttime);

            // 20s exactly!
            if res.ttime == Duration::from_secs(20) {
                context.queue_packet(Mtc {
                    text: format!(
                        "{} got exactly a 20s lap!",
                        "PLACEHOLDER", // FIXME
                    ),
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
                    context.state = GameState::Ended;

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

fn main() {
    Workshop::with_state(GameState::Waiting)
        .add_engine(Setup)
        .add_engine(CountdownEngine::new())
        .add_engine(Rounds::default())
        .ignition(insim::tcp("127.0.0.1:29999").set_non_blocking(true))
        .run(Duration::from_millis(1000 / 60));
}
