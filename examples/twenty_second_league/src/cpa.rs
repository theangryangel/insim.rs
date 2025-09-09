//! Clear penalties automatically
use std::fmt::Debug;

use insim::insim::{Mst, Pen};
use kitcar::{Context, Engine};

/// Prevent voting
#[derive(Debug)]
pub struct Cpa;

impl<S, P, C, G> Engine<S, P, C, G> for Cpa
where
    S: Default + Debug,
    P: Default + Debug,
    C: Default + Debug,
    G: Default + Debug,
{
    fn pen(&mut self, context: &mut Context<S, P, C, G>, packet: &Pen) {
        if_chain::if_chain! {
            if let Some(player) = context.players.get(&packet.plid);
            if let Some(connection) = context.connections.get(&player.ucid);
            if connection.uname.len() > 0;
            then {
                context.queue_packet(Mst {
                    msg: format!("/p_clear {}", &connection.uname),
                    ..Default::default()
                });
            }
        }
    }
}
