//! No voting!
use insim::insim::{TinyType, Vtn};
use kitcar::{Context, Engine};

use crate::{GameState, State};

/// Prevent voting when a game is in progress
#[derive(Debug)]
pub struct NoVote;

impl Engine<State> for NoVote {
    fn active(&self, context: &Context<State>) -> bool {
        matches!(context.state.inner, GameState::InProgress { .. })
    }

    fn vtn(&mut self, context: &mut Context<State>, _vtn: &Vtn) {
        context.queue_packet(TinyType::Vtc);
    }
}
