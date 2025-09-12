//! Twenty Second League implementation
use std::{fmt::Debug, time::Duration};

use insim::{
    identifiers::ConnectionId,
    insim::{Mso, Mst, Mtc, Res},
};
use kitcar::{Context, Engine, Timer};

use crate::combo::{Combo, ComboCollection};

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
    LoadTrack {
        combo: Combo,
        timer: Option<Timer>,
    },
    /// Main game loop is in progress.
    InGame {
        /// Chosen combo
        combo: Combo,

        /// Current round
        round: u8,
        /// Time until next round
        timer: Timer,
    },
}

struct CountdownView;

impl Engine<ComboCollection, (), (), ()> for League {
    fn tick(&mut self, context: &mut Context<ComboCollection, (), (), ()>) {
        match self {
            League::Lobby { countdown } => {
                if countdown.tick() {
                    if countdown.is_finished() {
                        for (_ucid, info) in context.connections.iter_mut() {
                            let _ = info.ui.remove_tree::<CountdownView>();
                        }

                        let combo = context.state.random().cloned().unwrap();

                        *self = Self::LoadTrack { timer: None, combo };
                    } else {
                        let remaining =
                            countdown.remaining_duration() * countdown.remaining_repeats().unwrap();

                        let countdown = crate::components::countdown(remaining);

                        for (_ucid, info) in context.connections.iter_mut() {
                            let _ = info.ui.set_tree::<CountdownView>(countdown.clone());
                        }
                    }
                }
            },
            League::LoadTrack { timer: None, combo } => {
                context.queue_packet(Mst {
                    msg: format!("/end"),
                    ..Default::default()
                });

                *self = League::LoadTrack {
                    timer: Some(Timer::once(Duration::from_secs(6))),
                    combo: combo.clone(),
                };
            },
            League::LoadTrack {
                timer: Some(timer),
                combo,
            } => {
                if timer.tick() {
                    if timer.is_finished() {
                        // FIXME: check game state
                    }

                    let iteration = timer.iteration().saturating_sub(1);

                    match iteration {
                        0 => {
                            context.queue_packet(Mst {
                                msg: format!("/end"),
                                ..Default::default()
                            });
                        },
                        1 => {
                            context.queue_packet(Mst {
                                msg: "/clear".into(),
                                ..Default::default()
                            });

                            context.queue_packet(Mst {
                                msg: format!("/track={}", combo.track),
                                ..Default::default()
                            });
                        },
                        2 => {
                            context.queue_packet(Mst {
                                msg: format!("/qual=0"),
                                ..Default::default()
                            });

                            context.queue_packet(Mst {
                                msg: format!("/laps={}", combo.laps.unwrap_or(1)),
                                ..Default::default()
                            });
                        },
                        3 => {
                            if let Some(layout) = combo.layout.as_ref() {
                                context.queue_packet(Mst {
                                    msg: format!("/layout={}", layout),
                                    ..Default::default()
                                });
                            }
                        },
                        _ => {},
                    }
                }
            },
            League::InGame {
                timer,
                ref mut round,
                ..
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

    fn mso(&mut self, context: &mut Context<ComboCollection, (), (), ()>, mso: &Mso) {
        let is_admin = context
            .connections
            .get(&mso.ucid)
            .map_or_else(|| false, |u| u.admin);

        println!("{:?} {:?}", mso, mso.msg_from_textstart());

        match mso.msg_from_textstart() {
            "!lobby" if matches!(&self, Self::Idle) && is_admin => {
                *self = Self::Lobby {
                    countdown: Timer::countdown(Duration::from_secs(15)),
                };
            },
            "!end" if is_admin => {
                *self = Self::Idle;
            },
            "!state" => {
                println!("{:?}", self);
                context.queue_packet(Mtc {
                    ucid: mso.ucid.clone(),
                    text: format!("{:?}", self),
                    ..Default::default()
                });
            },
            _ => {},
        }
    }

    fn res(&mut self, context: &mut Context<ComboCollection, (), (), ()>, res: &Res) {
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
