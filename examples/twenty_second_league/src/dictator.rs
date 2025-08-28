//! No voting!
use std::fmt::Debug;

use insim::insim::{TinyType, Vtn};
use kitcar::{Context, Engine};

/// Prevent voting
#[derive(Debug)]
pub struct NoVote;

impl<S, P, C> Engine<S, P, C> for NoVote
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
{
    fn vtn(&mut self, context: &mut Context<S, P, C>, _vtn: &Vtn) {
        context.queue_packet(TinyType::Vtc);
    }
}
