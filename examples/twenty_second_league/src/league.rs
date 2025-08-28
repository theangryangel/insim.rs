//! Twenty Second League implementation
use std::{fmt::Debug, time::Duration};

use insim::{
    identifiers::ConnectionId,
    insim::{Mal, Mso, Mst, Mtc, PlcAllowedCarsSet, Res, SmallType},
};
use kitcar::{Context, Engine, Timer};

/// Represents the state of the mini-game.
#[derive(Debug)]
pub enum League {
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

impl<S, P, C> Engine<S, P, C> for League
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
{
    fn tick(&mut self, context: &mut Context<S, P, C>) {
        match self {
            League::Lobby { countdown } => {
                if countdown.tick() {
                    if countdown.is_finished() {
                        *self = Self::InGame {
                            round: 0,
                            timer: Timer::repeating(Duration::from_secs(60), 15),
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
                        let remaining = countdown.remaining_duration();
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
            },
            League::InGame {
                timer,
                ref mut round,
            } => {
                if timer.tick() {
                    if *round > 0 {
                        // TODO store results
                    }
                    *round = *round + 1;

                    if timer.is_finished() {
                        *self = Self::Idle;

                        context.queue_packet(Mtc {
                            text: "The game has ended! Thanks for playing".to_owned(),
                            ucid: ConnectionId::ALL,
                            ..Default::default()
                        });
                        // TODO: announce winners, etc.
                        return;
                    }

                    // context.queue_packet(Mst {
                    //     msg: format!("/laps {}", context.state.combo.as_ref().unwrap().laps),
                    //     ..Default::default()
                    // });
                    // context.queue_packet(Mst {
                    //     msg: "/restart".to_owned(),
                    //     ..Default::default()
                    // });
                    // context.queue_packet(Mtc {
                    //     text: format!("Round {} is now starting! Good luck!", round),
                    //     ucid: ConnectionId::ALL,
                    //     ..Default::default()
                    // });
                }
            },
            _ => {},
        }
    }

    fn mso(&mut self, _context: &mut Context<S, P, C>, mso: &Mso) {
        // FIXME: check if admin
        match mso.msg.as_str() {
            "!lobby" if matches!(&self, Self::Idle) => {
                *self = Self::Lobby {
                    countdown: Timer::repeating(Duration::from_secs(10), 30),
                };
            },
            "!end" => {
                *self = Self::Idle;
            },
            _ => {},
        }
    }

    fn res(&mut self, context: &mut Context<S, P, C>, res: &Res) {
        if !matches!(self, Self::InGame { .. }) {
            return;
        }

        // let _ = self.results.insert(res.uname.clone(), res.ttime);

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
