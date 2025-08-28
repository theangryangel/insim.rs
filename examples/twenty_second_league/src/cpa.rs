//! Clear penalties automatically
use std::fmt::Debug;

use insim::insim::{Mst, Pen, TinyType};
use kitcar::{Context, Engine};

/// Prevent voting
#[derive(Debug)]
pub struct Cpa;

impl<S, P, C> Engine<S, P, C> for Cpa
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
{
    fn pen(&mut self, context: &mut Context<S, P, C>, packet: &Pen) {
        context.queue_packet(Mst {
            msg: format!("/p_clear {}", packet.plid),
            ..Default::default()
        });
    }
}
