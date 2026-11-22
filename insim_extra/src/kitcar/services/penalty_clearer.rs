//! [`PenaltyClearer`]: clears each penalty after a fixed delay.
//!
//! Start it next to your game. It needs nothing from the game, so its messages
//! and timers stay out of the game's way.
//!
//! ```ignore
//! let _game = host.start(MyGame::new()).await?;
//! let _clearer = host.start(PenaltyClearer::new(Duration::from_secs(15))).await?;
//! ```

use std::time::Duration;

use insim::{Packet, identifiers::ConnectionId, insim::PenaltyReason};

use crate::{
    kitcar::{Context, Effects, Input, Service},
    world::WorldEvent,
};

/// A connection's clear is due. The [`PenaltyClearer`]'s own message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClearPenalty(ConnectionId);

/// One pending clear per connection, so a new penalty restarts the delay.
#[derive(Debug, PartialEq)]
struct Pending(ConnectionId);

/// Clears each penalty `delay` after it is given. A new penalty restarts the
/// delay. A session starting or ending drops pending clears.
#[derive(Debug, Clone, Copy)]
pub struct PenaltyClearer {
    delay: Duration,
}

impl PenaltyClearer {
    /// Clear penalties after `delay`.
    pub fn new(delay: Duration) -> Self {
        Self { delay }
    }
}

impl Service for PenaltyClearer {
    type Msg = ClearPenalty;

    fn update(&mut self, input: Input<ClearPenalty>, cx: &Context<'_>) -> Effects<ClearPenalty> {
        match input {
            Input::Packet(Packet::Pen(pen)) if !matches!(pen.reason, PenaltyReason::Unknown) => {
                match cx.world().connection_by_player(pen.plid) {
                    Some(conn) => Effects::new().schedule_after(
                        Pending(conn.ucid),
                        self.delay,
                        ClearPenalty(conn.ucid),
                    ),
                    None => Effects::none(),
                }
            },
            Input::World(WorldEvent::SessionStarted(_) | WorldEvent::SessionEnded(_)) => {
                Effects::new().cancel_all::<Pending>()
            },
            Input::Msg(ClearPenalty(ucid)) => match cx.world().connection(ucid) {
                Some(conn) => Effects::new().packet(conn.clear_penalty()),
                None => Effects::none(),
            },
            _ => Effects::none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use insim::{
        identifiers::PlayerId,
        insim::{Ncn, Npl, Pen},
    };

    use super::*;
    use crate::world::World;

    #[test]
    fn a_penalty_schedules_one_clear_that_sends_p_clear() {
        let mut world = World::new();
        let _ = world.apply_packet(
            &Ncn {
                ucid: ConnectionId(4),
                uname: "driver".into(),
                ..Default::default()
            }
            .into(),
        );
        let _ = world.apply_packet(
            &Npl {
                ucid: ConnectionId(4),
                plid: PlayerId(9),
                nump: 1,
                ..Default::default()
            }
            .into(),
        );
        let cx = Context::new(&world, Instant::now());
        let mut clearer = PenaltyClearer::new(Duration::from_secs(15));
        let pen = Input::Packet(
            Pen {
                plid: PlayerId(9),
                reason: PenaltyReason::FalseStart,
                ..Default::default()
            }
            .into(),
        );
        let effects = clearer.update(pen, &cx);
        let timers: Vec<_> = effects.timers().map(|(_, msg)| *msg).collect();
        assert_eq!(timers, [ClearPenalty(ConnectionId(4))]);

        let cleared = clearer.update(Input::Msg(ClearPenalty(ConnectionId(4))), &cx);
        assert!(matches!(
            cleared.packets_sent().next(),
            Some(Packet::Mst(mst)) if mst.msg == "/p_clear driver"
        ));
    }
}
