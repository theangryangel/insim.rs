//! Redrawing services' views, at most once a frame.

use std::time::Instant;

use insim::Packet;

use super::{FRAME, Loop, Target, context};
use crate::timers;

impl Loop {
    /// When to next redraw: a frame after the last if anything changed, or
    /// when a view asked to be drawn again.
    pub(super) fn render_at(&self) -> Option<Instant> {
        let frame = self.last_render + FRAME;
        if self.services.iter().any(|s| s.service.needs_render()) {
            return Some(frame);
        }
        self.services
            .iter()
            .filter_map(|s| s.service.redraw_at())
            .min()
            .map(|at| at.max(frame))
    }

    pub(super) async fn render(&mut self) {
        let now = timers::now();
        self.last_render = now;
        let mut packets = Vec::new();
        // What components reported as they mounted, for their services.
        let mut raised = Vec::new();
        {
            let cx = context(&self.world, self.director.as_ref(), now);
            for (index, running) in self.services.iter_mut().enumerate() {
                let (drawn, effects) = running.service.render(&cx, &mut self.pools);
                packets.extend(drawn);
                if !effects.is_empty() {
                    raised.push((index, effects));
                }
            }
        }
        self.send_all(packets).await;
        for (index, effects) in raised {
            self.apply(Target::Service(index), effects, now).await;
        }
        self.direct(now).await;
    }

    pub(super) async fn send_all(&mut self, packets: Vec<Packet>) {
        for packet in packets {
            if let Err(error) = self.connection.send(packet).await {
                tracing::warn!(%error, "failed to send buttons");
            }
        }
    }
}
