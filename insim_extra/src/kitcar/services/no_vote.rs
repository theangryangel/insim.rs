//! [`NoVote`]: cancels players' votes for chosen actions.
//!
//! Start it next to your game, or next to a
//! [`RaceDirector`](crate::kitcar::director::RaceDirector) whose restarts and
//! rotation a vote would otherwise cut short.
//!
//! ```ignore
//! let _game = host.start(MyGame::new()).await?;
//! let _votes = host
//!     .start(
//!         NoVote::new([VtnAction::End, VtnAction::Restart])
//!             .message("^1Votes to end or restart are off here"),
//!     )
//!     .await?;
//! ```

use std::convert::Infallible;

use insim::{
    Packet,
    insim::{TinyType, Vtn, VtnAction},
};

use crate::kitcar::{Context, Effects, Input, Service};

/// Cancels every vote for one of its actions as soon as LFS reports it
/// (`/vote cancel`, as `TINY_VTC`). Votes for anything else go ahead.
#[derive(Debug, Clone)]
pub struct NoVote {
    actions: Vec<VtnAction>,
    message: Option<String>,
}

impl NoVote {
    /// Cancel votes for these actions, such as [`VtnAction::Restart`].
    pub fn new(actions: impl IntoIterator<Item = VtnAction>) -> Self {
        Self {
            actions: actions.into_iter().collect(),
            message: None,
        }
    }

    /// Tell whoever voted why their vote vanished.
    pub fn message(mut self, text: impl Into<String>) -> Self {
        self.message = Some(text.into());
        self
    }
}

impl Service for NoVote {
    type Msg = Infallible;

    fn update(&mut self, input: Input<Infallible>, _: &Context<'_>) -> Effects<Infallible> {
        match input {
            // `None` is a vote being withdrawn or cancelled: nothing to stop.
            Input::Packet(Packet::Vtn(Vtn { action, ucid, .. }))
                if action != VtnAction::None && self.actions.contains(&action) =>
            {
                tracing::info!(?action, %ucid, "cancelling a vote");
                let effects = Effects::new().packet(TinyType::Vtc);
                match &self.message {
                    Some(text) => effects.message(text.clone(), ucid),
                    None => effects,
                }
            },
            _ => Effects::none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use insim::{identifiers::ConnectionId, insim::Tiny};

    use super::*;
    use crate::world::World;

    fn vote(action: VtnAction) -> Input<Infallible> {
        Input::Packet(
            Vtn {
                ucid: ConnectionId(3),
                action,
                ..Default::default()
            }
            .into(),
        )
    }

    #[test]
    fn cancels_only_the_chosen_votes() {
        let world = World::new();
        let cx = Context::new(&world, Instant::now());
        let mut no_vote = NoVote::new([VtnAction::Restart, VtnAction::Qualify]);

        for action in [VtnAction::Restart, VtnAction::Qualify] {
            let effects = no_vote.update(vote(action), &cx);
            let sent: Vec<_> = effects.packets_sent().collect();
            assert!(
                matches!(
                    sent[..],
                    [Packet::Tiny(Tiny {
                        subt: TinyType::Vtc,
                        ..
                    })]
                ),
                "{sent:?}"
            );
        }
        assert!(no_vote.update(vote(VtnAction::End), &cx).is_empty());
        assert!(no_vote.update(vote(VtnAction::None), &cx).is_empty());
    }

    #[test]
    fn a_withdrawn_vote_is_never_cancelled() {
        let world = World::new();
        let cx = Context::new(&world, Instant::now());
        let mut no_vote = NoVote::new([VtnAction::None]);
        assert!(no_vote.update(vote(VtnAction::None), &cx).is_empty());
    }

    #[test]
    fn tells_the_voter_why() {
        let world = World::new();
        let cx = Context::new(&world, Instant::now());
        let mut no_vote = NoVote::new([VtnAction::Restart]).message("no restarts");
        let effects = no_vote.update(vote(VtnAction::Restart), &cx);
        let sent: Vec<_> = effects.packets_sent().collect();
        assert!(
            matches!(
                sent[..],
                [Packet::Tiny(Tiny { subt: TinyType::Vtc, .. }), Packet::Mtc(mtc)]
                    if mtc.ucid == ConnectionId(3) && mtc.text == "no restarts"
            ),
            "{sent:?}"
        );
    }
}
