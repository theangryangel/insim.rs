//! [`PenaltyClearer`]: clears a penalty a fixed time after it is given.
//!
//! A component: hold it as a field of your service, pass it every input, and route
//! its message back to it.
//!
//! ```ignore
//! enum Msg { Penalty(ClearPenalty), /* ... */ }
//!
//! fn update(&mut self, input: Input<Msg>, cx: &Context) -> Effects<Msg> {
//!     let fx = self.clearer.update(&input, cx).map(Msg::Penalty);
//!     fx.and(match input {
//!         Input::Msg(Msg::Penalty(clear)) => self.clearer.clear(clear, cx),
//!         // ...
//!         _ => Effects::none(),
//!     })
//! }
//! ```

use std::time::Duration;

use insim::{Packet, identifiers::ConnectionId, insim::PenaltyReason};
use insim_extra::world::WorldEvent;

use crate::{Context, Effects, Input};

/// The clear for one connection is due. Hand it to [`PenaltyClearer::clear`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClearPenalty(pub ConnectionId);

#[derive(Debug, PartialEq)]
struct Pending(ConnectionId);

/// Clears each penalty `delay` after it is given. A new penalty restarts the
/// delay, and pending clears are dropped when a session starts or ends.
#[derive(Debug, Clone, Copy)]
pub struct PenaltyClearer {
    delay: Duration,
}

impl PenaltyClearer {
    /// Clear penalties `delay` after they are given.
    pub fn new(delay: Duration) -> Self {
        Self { delay }
    }

    /// Schedule or discard clears. Pass every input.
    pub fn update<M>(&mut self, input: &Input<M>, cx: &Context<'_>) -> Effects<ClearPenalty> {
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
            _ => Effects::none(),
        }
    }

    /// Carry out a due clear, if the connection is still there.
    pub fn clear<M: Send + 'static>(&self, due: ClearPenalty, cx: &Context<'_>) -> Effects<M> {
        match cx.world().connection(due.0) {
            Some(conn) => Effects::new().packet(conn.clear_penalty()),
            None => Effects::none(),
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
    use insim_extra::world::World;

    use super::*;

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
        let pen = Input::<()>::Packet(
            Pen {
                plid: PlayerId(9),
                reason: PenaltyReason::FalseStart,
                ..Default::default()
            }
            .into(),
        );
        let effects = clearer.update(&pen, &cx);
        let timers: Vec<_> = effects.timers().map(|(_, msg)| *msg).collect();
        assert_eq!(timers, [ClearPenalty(ConnectionId(4))]);

        let cleared: Effects<()> = clearer.clear(ClearPenalty(ConnectionId(4)), &cx);
        assert!(matches!(
            cleared.packets_sent().next(),
            Some(Packet::Mst(mst)) if mst.msg == "/p_clear driver"
        ));
    }
}
