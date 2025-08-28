//! Twenty Second League implementation
use std::{fmt::Debug, time::Duration};

use insim::{
    core::string::colours::Colourify,
    identifiers::{ClickId, ConnectionId},
    insim::{Bfn, Btn, BtnStyle, Mal, Mso, Mst, Mtc, PlcAllowedCarsSet, Res, SmallType, TinyType},
    WithRequestId,
};
use kitcar::{Context, Engine, Timer};

/// Represents the state of the mini-game.
#[derive(Debug)]
pub struct Countdown {
    countdown: Timer,
}

impl Countdown {
    pub fn new() -> Self {
        Self {
            countdown: Timer::countdown(Duration::from_secs(15)),
        }
    }
}

impl<S, P, C> Engine<S, P, C> for Countdown
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
{
    fn tick(&mut self, context: &mut Context<S, P, C>) {
        if self.countdown.tick() {
            if self.countdown.is_finished() {
                context.queue_packet(Bfn {
                    ucid: ConnectionId::ALL,
                    subt: insim::insim::BfnType::Clear,
                    ..Default::default()
                });

                context.queue_packet(Mst {
                    msg: format!("/end"),
                    ..Default::default()
                });

                context.queue_packet(Mst {
                    msg: format!("/track FE1"),
                    ..Default::default()
                });
            } else {
                let remaining = self.countdown.remaining_duration()
                    * self.countdown.remaining_repeats().unwrap();
                let seconds = remaining.as_secs() % 60;
                let minutes = (remaining.as_secs() / 60) % 60;

                // Background
                context.queue_packet(
                    Btn {
                        ucid: ConnectionId::ALL,
                        clickid: ClickId(1),
                        bstyle: BtnStyle::default().light(),
                        w: 22,
                        h: 22,
                        l: 10,
                        t: 115,

                        ..Default::default()
                    }
                    .with_request_id(1),
                );

                // Foreground - welcome
                context.queue_packet(
                    Btn {
                        text: "Welcome to 20sl.".to_owned(),
                        ucid: ConnectionId::ALL,
                        clickid: ClickId(2),
                        bstyle: BtnStyle::default().align_left().black(),
                        w: 20,
                        h: 5,
                        l: 11,
                        t: 116,

                        ..Default::default()
                    }
                    .with_request_id(2),
                );

                let mut bstyle = BtnStyle::default().light().black();
                if seconds < 10 {
                    bstyle = bstyle.red();
                }

                context.queue_packet(
                    Btn {
                        text: format!("{:02}:{:02}", minutes, seconds),
                        ucid: ConnectionId::ALL,
                        clickid: ClickId(3),
                        bstyle,
                        w: 20,
                        h: 10,
                        l: 11,
                        t: 121,

                        ..Default::default()
                    }
                    .with_request_id(3),
                );

                // Foreground - game starts in
                context.queue_packet(
                    Btn {
                        text: "..until game starts".to_owned(),
                        ucid: ConnectionId::ALL,
                        clickid: ClickId(4),
                        bstyle: BtnStyle::default().align_left().black(),
                        w: 20,
                        h: 5,
                        l: 11,
                        t: 131,

                        ..Default::default()
                    }
                    .with_request_id(4),
                );
            }
        }
    }
}
