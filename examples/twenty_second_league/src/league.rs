//! Twenty Second League implementation
use std::{fmt::Debug, time::Duration};

use insim::{
    identifiers::ConnectionId,
    insim::{BtnStyle, Mal, Mso, Mst, Mtc, PlcAllowedCarsSet, Res, SmallType},
};
use kitcar::{
    ui::components::{basic, fullscreen, vstack},
    Context, Engine, Timer,
};

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
    LoadTrack,
    Warmup,
    /// Main game loop is in progress.
    InGame {
        /// Current round
        round: u8,
        /// Time until next round
        timer: Timer,
    },
}

struct CountdownView;

impl<S, P, C, G> Engine<S, P, C, G> for League
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
    G: Default + Debug,
{
    fn tick(&mut self, context: &mut Context<S, P, C, G>) {
        match self {
            League::Lobby { countdown } => {
                if countdown.tick() {
                    if countdown.is_finished() {
                        for (_ucid, info) in context.connections.iter_mut() {
                            let _ = info.ui.remove_tree::<CountdownView>();
                        }

                        // TODO: end game, load track, load layout
                        *self = Self::LoadTrack;
                    } else {
                        let remaining =
                            countdown.remaining_duration() * countdown.remaining_repeats().unwrap();
                        let seconds = remaining.as_secs() % 60;
                        let minutes = (remaining.as_secs() / 60) % 60;

                        let countdown = fullscreen()
                            .height(150.0)
                            .display_flex()
                            .flex_direction_column()
                            .align_items_flex_start()
                            .justify_content_flex_start()
                            .padding(20.0)
                            .with_child(
                                kitcar::ui::node::UINode::rendered(
                                    BtnStyle::default().dark(),
                                    "", 
                                    1.into()
                                )
                                .display_block()
                                .position_relative()
                                .padding(1.0)
                                .with_children(
                                    [basic(
                                        "Welcome to ^120sl^8, game starts in".into(),
                                        35,
                                        5,
                                        2.into(),
                                    ),
                                    basic(
                                        format!("{:02}:{:02}", minutes, seconds).into(),
                                        35,
                                        15,
                                        3.into(),
                                    )]
                                )
                            );

                        for (_ucid, info) in context.connections.iter_mut() {
                            let _ = info.ui.set_tree::<CountdownView>(countdown.clone());
                        }
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

    fn mso(&mut self, context: &mut Context<S, P, C, G>, mso: &Mso) {
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

    fn res(&mut self, context: &mut Context<S, P, C, G>, res: &Res) {
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
