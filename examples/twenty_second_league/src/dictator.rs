//! No voting!
use std::fmt::Debug;

use insim::insim::{TinyType, Vtn};
use kitcar::{Context, Engine};

/// Prevent voting
#[derive(Debug)]
pub struct NoVote;

impl<S, P, C, G> Engine<S, P, C, G> for NoVote
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
    G: Default + Debug,
{
    fn vtn(&mut self, context: &mut Context<S, P, C, G>, _vtn: &Vtn) {
        context.queue_packet(TinyType::Vtc);
    }
}
