//! [`RaceDirector`]: runs the races most mini-games share. It waits for
//! players, sets up the lobby, races, and moves through a [`Rotation`] of combos
//! (track, layout, cars).
//!
//! It isn't a [`Service`](crate::kitcar::Service): like the [`World`], the
//! [`Host`](crate::kitcar::Host) owns it, so there is never more than one
//! fighting over `/restart`. The host runs it before every service, and
//! services read it with [`Context::director`] to show players what's coming.
//! Games still follow the session in [`World`], which reports races starting
//! and ending whoever caused them: the director, an admin's `/end` or
//! `/restart`, or everyone leaving.
//!
//! ```ignore
//! // A new combo every 3 races, in order.
//! let director = RaceDirector::new(Cycle::new([fe1x.races(3), bl1.races(3)]));
//! host.start_director(director).await?;
//! let _game = host.start(MyGame::new()).await?;
//! ```
//!
//! Pass one [`Combo`] to keep it forever, or a [`Cycle`] to loop through
//! several, in order or reshuffled each pass. For votes or layouts from disk,
//! pass a closure; see [`Rotation`].

mod combo;
mod rotation;
mod setup;
#[cfg(test)]
mod tests;

use std::{
    fmt,
    time::{Duration, Instant},
};

pub use combo::{Combo, Layout, StartOrder};
use insim::{identifiers::ConnectionId, insim::Mst};
pub use rotation::{Cycle, Rotation, RotationError};

use crate::{
    kitcar::{Context, Effects, Input},
    world::{
        World, WorldEvent,
        race::{FinishStatus, RaceEvent},
    },
};

/// Where the director's round is. Internal: services read what only the
/// director knows from [`RaceDirectorStatus`], and the rest (racing or in the
/// lobby) from the [`World`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Waiting for players, or for the lobby.
    #[default]
    Waiting,
    /// Setting up the lobby. Players type `/ready` to start.
    SettingUp,
    /// A race is running.
    Racing,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Waiting => "waiting for players",
            Self::SettingUp => "setting up track",
            Self::Racing => "racing",
        })
    }
}

/// What a [`RaceDirector`] is doing, as services see it through
/// [`Context::director`](crate::kitcar::Context::director). Plain data, so a
/// test can build one with `..Default::default()` and hand it to
/// [`Context::with_director`](crate::kitcar::Context::with_director).
#[derive(Debug, Clone, Copy, Default)]
pub struct RaceDirectorStatus<'a> {
    /// The lobby is set up for [`combo`](Self::combo): players can type
    /// `/ready`. False while a track changes or the next combo loads, which
    /// the [`World`] alone can't tell from a lobby that's ready.
    pub ready: bool,
    /// The combo set up on the server now, so it matches the track being
    /// raced even while the next one loads. `None` until the first is.
    pub combo: Option<&'a Combo>,
    /// What the rotation says comes after, if it can say. See
    /// [`Rotation::peek`].
    pub next: Option<&'a Combo>,
    /// With [`Combo::races`] set on the combo being raced, the races ended on
    /// it and how many it gets: `(1, 3)` while the second of three runs.
    pub races: Option<(u32, u32)>,
    /// When the finished race restarts (or returns to the lobby), while the
    /// auto-restart grace period runs.
    pub restart_at: Option<Instant>,
    /// Whether the next combo is loading.
    pub loading: bool,
}

/// A change of phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Transition {
    /// The lobby is being set up.
    SetupStarted,
    /// The race started.
    Racing,
    /// The race ended. `shortfall` if everyone left, not the session.
    Ended { shortfall: bool },
}

/// Steers the [`RaceDirector`] from a service, with
/// [`Effects::director`](crate::kitcar::Effects::director).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum DirectorCommand {
    /// Move on to the next combo now, ending the running race. Ignored while
    /// the next combo is already loading.
    Rotate,
    /// Race this combo next, now, ending the running race: a vote's winner, or
    /// an admin's pick. Replaces a combo still loading. The rotation carries
    /// on afterwards as if this had never come up.
    Next(Combo),
}

/// The [`RaceDirector`]'s own messages: its timers, and combos as they load.
#[derive(Debug, Clone)]
pub(crate) struct RaceDirectorMsg(DirectorMsg);

#[derive(Debug, Clone)]
enum DirectorMsg {
    /// The rotation produced the next combo, or failed to.
    Loaded(Result<Combo, String>),
    /// The grace period after the first finisher is over.
    GraceUp,
}

/// The grace period after the first finisher.
#[derive(Debug, PartialEq)]
struct Grace;

/// The rotation's next combo, while it loads.
#[derive(Debug, PartialEq)]
struct Load;

/// Once everyone is home, the longest the next race waits, so the results
/// stay up for a moment and LFS has time to confirm them.
const SETTLE: Duration = Duration::from_secs(10);

const READY: &str = "^2Setup complete - type ^3/ready^2 to start!";
const LOADING: &str = "^3Loading the next track, please wait...";

/// Runs the races most mini-games share. Waits for players, sets up the lobby
/// with the current [`Combo`], races, and moves on through its [`Rotation`].
///
/// Multiplayer only, like the rest of kitcar. It counts players with
/// [`World::connection_count`], which leaves out the server's own connection.
pub struct RaceDirector {
    rotation: Box<dyn Rotation>,
    /// `None` until the first combo loads.
    current: Option<Combo>,
    /// The combo last set up on the server, which may be older than `current`
    /// between a rotation and its lobby.
    set_up: Option<Combo>,
    /// The lobby must wait while a combo loads.
    loading: bool,
    phase: Phase,
    configured: bool,
    ending_session: bool,
    /// Races ended on the current combo, for [`Combo::races`].
    races: u32,
    /// The grace period after the first finisher, if auto-restarting.
    auto_restart: Option<Duration>,
    /// When the grace period after the first finisher ends, while it runs.
    restart_at: Option<Instant>,
    /// `/restart` or `/end` is sent; ignore finishes until the session ends.
    moved_on: bool,
    /// Picks each combo's time of day.
    rng: fastrand::Rng,
}

impl fmt::Debug for RaceDirector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RaceDirector")
            .field("current", &self.current)
            .field("loading", &self.loading)
            .field("phase", &self.phase)
            .field("configured", &self.configured)
            .field("races", &self.races)
            .finish_non_exhaustive()
    }
}

impl RaceDirector {
    /// Direct races with combos from `rotation`: a [`Combo`], a [`Cycle`] or a
    /// closure. Stays on each combo for its [`Combo::races`], or for good
    /// without.
    pub fn new(rotation: impl Rotation) -> Self {
        Self {
            rotation: Box::new(rotation),
            current: None,
            set_up: None,
            loading: false,
            phase: Phase::Waiting,
            configured: false,
            ending_session: false,
            races: 0,
            auto_restart: None,
            restart_at: None,
            moved_on: false,
            rng: fastrand::Rng::new(),
        }
    }

    /// Seed the random picks (a combo's time of day), for repeatable runs.
    pub fn seed(mut self, seed: u64) -> Self {
        self.rng = fastrand::Rng::with_seed(seed);
        self
    }

    /// Restart finished races for the players, who would otherwise wait on the
    /// results screen for an admin. The first finisher starts a `grace`
    /// period. Once everyone is home it's cut to 10 seconds, and players are
    /// told. When it runs out the race restarts, or returns to the lobby if
    /// the next combo is due.
    pub fn auto_restart(mut self, grace: Duration) -> Self {
        self.auto_restart = Some(grace);
        self
    }

    /// What services see of the director through
    /// [`Context::director`](crate::kitcar::Context::director).
    pub fn status(&self) -> RaceDirectorStatus<'_> {
        RaceDirectorStatus {
            ready: self.phase == Phase::SettingUp && self.configured && !self.loading,
            combo: self.set_up.as_ref(),
            next: self.rotation.peek(),
            races: self.races(),
            restart_at: self.restart_at,
            loading: self.loading,
        }
    }

    /// Where the round is.
    #[cfg(test)]
    pub(crate) fn phase(&self) -> Phase {
        self.phase
    }

    /// The latest combo loaded, set up or not.
    #[cfg(test)]
    pub(crate) fn combo(&self) -> Option<&Combo> {
        self.current.as_ref()
    }

    /// Whether a race is running.
    #[cfg(test)]
    pub(crate) fn racing(&self) -> bool {
        self.phase == Phase::Racing
    }

    /// With [`Combo::races`] set on the combo being raced, the races ended on
    /// it and how many it gets: `(1, 3)` while the second of three runs.
    fn races(&self) -> Option<(u32, u32)> {
        let of = self.set_up.as_ref()?.races?;
        Some((self.races, of))
    }

    /// Whether the next combo is loading.
    #[cfg(test)]
    pub(crate) fn loading(&self) -> bool {
        self.loading
    }

    /// Ask the rotation for the next combo. It arrives as a message.
    fn load_next(&mut self, world: &World) -> Effects<RaceDirectorMsg> {
        tracing::info!("loading the next combo");
        self.loading = true;
        let next = self.rotation.next(self.current.as_ref(), world);
        Effects::new().keyed_task(Load, async move {
            RaceDirectorMsg(DirectorMsg::Loaded(
                next.await.map_err(|err| err.to_string()),
            ))
        })
    }

    /// Use the new combo, and set up the lobby now if we're in one. On failure,
    /// stay on the current combo.
    fn loaded(&mut self, result: Result<Combo, String>, world: &World) -> Effects<RaceDirectorMsg> {
        self.loading = false;
        self.races = 0;
        let effects = Effects::new();
        let effects = match result {
            Ok(combo) => {
                let first = self.current.is_none();
                tracing::info!(track = %combo.track, name = ?combo.name, "combo loaded");
                let announce = format!("^3Next track: ^7{}", combo.display_name());
                self.current = Some(combo);
                self.configured = false;
                if first {
                    effects
                } else {
                    effects.message(announce, ConnectionId::ALL)
                }
            },
            Err(err) => {
                tracing::warn!(error = %err, "rotation failed to produce a combo");
                match &self.current {
                    Some(combo) => effects.message(
                        format!(
                            "^1Couldn't load the next track - staying on ^7{}",
                            combo.display_name()
                        ),
                        ConnectionId::ALL,
                    ),
                    None => effects,
                }
            },
        };
        if self.phase == Phase::SettingUp && world.session().is_none() {
            effects.and(self.set_up(world))
        } else {
            effects
        }
    }

    /// As a race ends, count it and move on if it was the combo's last.
    fn follow(&mut self, transition: Transition, cx: &Context<'_>) -> Effects<RaceDirectorMsg> {
        if transition != (Transition::Ended { shortfall: false }) {
            return Effects::none();
        }
        // Asked before counting it: was the race that ended the last?
        let last = self.last_race();
        self.races += 1;
        if last && !self.loading {
            self.load_next(cx.world())
        } else {
            Effects::none()
        }
    }

    /// Whether the race running (or next, in the lobby) is the combo's last.
    fn last_race(&self) -> bool {
        self.races().is_some_and(|(done, of)| done + 1 >= of)
    }

    /// With auto-restart on, start the grace period at the first finisher.
    /// Once everyone still on track is home, cut what's left of it to
    /// [`SETTLE`] and say so, rather than restarting under the results.
    fn watch_finish(
        &mut self,
        input: &Input<RaceDirectorMsg>,
        cx: &Context<'_>,
    ) -> Effects<RaceDirectorMsg> {
        let Some(grace) = self.auto_restart else {
            return Effects::none();
        };
        if self.phase != Phase::Racing {
            self.restart_at = None;
            self.moved_on = false;
            return Effects::none();
        }
        if self.moved_on {
            return Effects::none();
        }
        let finished = matches!(
            input.world(),
            Some(WorldEvent::Race(RaceEvent::Finished { .. }))
        );
        let first = finished && self.restart_at.is_none();
        if first {
            tracing::info!(?grace, "first finisher, grace period started");
            self.restart_at = Some(cx.now() + grace);
        }
        let Some(restart_at) = self.restart_at else {
            return Effects::none();
        };
        let home = cx
            .world()
            .live_entrants()
            .all(|entrant| matches!(entrant.status, FinishStatus::Finished { .. }));
        let settled = cx.now() + SETTLE;
        if home && settled < restart_at {
            tracing::info!(?SETTLE, "everyone is home, cutting the grace period");
            self.restart_at = Some(settled);
            return Effects::new()
                .schedule(Grace, settled, RaceDirectorMsg(DirectorMsg::GraceUp))
                .message(
                    format!(
                        "^3All drivers finished - next race in {}s",
                        SETTLE.as_secs()
                    ),
                    ConnectionId::ALL,
                );
        }
        if first {
            Effects::new()
                .schedule(Grace, restart_at, RaceDirectorMsg(DirectorMsg::GraceUp))
                .message(
                    format!("^3Next race in {}s", grace.as_secs()),
                    ConnectionId::ALL,
                )
        } else {
            Effects::none()
        }
    }

    /// Restart, or return to the lobby if the next combo is due.
    fn move_on(&mut self) -> Effects<RaceDirectorMsg> {
        self.restart_at = None;
        self.moved_on = true;
        let next = if self.last_race() || self.loading || !self.configured {
            tracing::info!("ending the race for the next combo");
            Mst::new("/end")
        } else {
            tracing::info!("restarting the race");
            Mst::new("/restart")
        };
        Effects::new().cancel(Grace).packet(next)
    }

    /// Act on a service's [`DirectorCommand`]. The host calls this.
    pub(crate) fn command(
        &mut self,
        command: DirectorCommand,
        cx: &Context<'_>,
    ) -> Effects<RaceDirectorMsg> {
        match command {
            DirectorCommand::Rotate => {
                if self.loading {
                    tracing::info!("rotation requested, but the next combo is already loading");
                    return Effects::none();
                }
                tracing::info!("rotation requested");
                self.rotate_now(cx.world())
            },
            DirectorCommand::Next(combo) => {
                tracing::info!(track = %combo.track, "next combo requested");
                // Any combo still loading would land after this one.
                let effects = Effects::new().cancel(Load);
                effects
                    .and(self.end_race())
                    .and(self.loaded(Ok(combo), cx.world()))
            },
        }
    }

    /// Load the next combo, ending the race if one is running. The lobby that
    /// follows waits for it.
    fn rotate_now(&mut self, world: &World) -> Effects<RaceDirectorMsg> {
        self.end_race().and(self.load_next(world))
    }

    /// End the running race to set up the next combo. It leaves
    /// [`Phase::Racing`] now, so the session ending isn't a race that counts
    /// towards the next combo's [`Combo::races`].
    fn end_race(&mut self) -> Effects<RaceDirectorMsg> {
        if self.phase != Phase::Racing {
            return Effects::none();
        }
        self.phase = Phase::SettingUp;
        self.ending_session = true;
        Effects::new().packet(Mst::new("/end"))
    }

    /// Advance the round by one input. Returns packets to send and any
    /// transition.
    fn round<M: Send + 'static>(
        &mut self,
        input: &Input<M>,
        cx: &Context<'_>,
    ) -> (Effects<M>, Option<Transition>) {
        let world = cx.world();
        match input.world() {
            // Any source counts: our setup, an admin's `/restart`, another
            // service. A mid-race `/restart` arrives as `SessionEnded` then
            // `SessionStarted`, so one round ends and the next starts.
            Some(WorldEvent::SessionStarted(_))
                if self.phase != Phase::Racing && world.connection_count() > 0 =>
            {
                tracing::info!(track = ?world.track(), "race started");
                self.phase = Phase::Racing;
                return (Effects::none(), Some(Transition::Racing));
            },
            Some(WorldEvent::SessionEnded(_)) if self.phase == Phase::Racing => {
                tracing::info!("race ended");
                self.phase = Phase::Waiting;
                return (
                    Effects::none(),
                    Some(Transition::Ended { shortfall: false }),
                );
            },
            Some(WorldEvent::Disconnected(_)) if world.connection_count() == 0 => {
                tracing::info!("everyone left, waiting for players");
                let was = self.phase;
                self.phase = Phase::Waiting;
                return match was {
                    Phase::Racing => (
                        Effects::new().packet(Mst::new("/end")),
                        Some(Transition::Ended { shortfall: true }),
                    ),
                    _ => (Effects::none(), None),
                };
            },
            _ => {},
        }

        // Until the startup Sta arrives, an unknown session doesn't prove we're
        // in the lobby. Reconcile from the settled world, not the event alone.
        let lobby_ready = matches!(input.world(), Some(WorldEvent::LobbyReady(_)));
        let track_changed = matches!(input.world(), Some(WorldEvent::TrackChanged(_)));
        let in_lobby =
            world.track().is_some() && world.session().is_none() && world.connection_count() > 0;
        if (lobby_ready || track_changed)
            && world.track().is_some()
            && world.session().is_none()
            && world.connection_count() == 0
        {
            tracing::info!("in the lobby, waiting for players");
        }
        if in_lobby && (lobby_ready || (!self.configured && track_changed)) {
            tracing::info!(track = ?world.track(), "setting up the lobby");
            self.ending_session = false;
            self.phase = Phase::SettingUp;
            return (self.set_up(world), Some(Transition::SetupStarted));
        }

        // Taking over mid-session: end it once and let the next lobby drive
        // setup.
        if !self.configured
            && !self.ending_session
            && world.session().is_some()
            && world.connection_count() > 0
        {
            tracing::info!("taking over mid-session, ending it to set up");
            self.ending_session = true;
            self.phase = Phase::SettingUp;
            return (
                Effects::new().packet(Mst::new("/end")),
                Some(Transition::SetupStarted),
            );
        }
        (Effects::none(), None)
    }
}

impl RaceDirector {
    /// Advance by one input. The host calls this before any service sees it.
    pub(crate) fn update(
        &mut self,
        input: Input<RaceDirectorMsg>,
        cx: &Context<'_>,
    ) -> Effects<RaceDirectorMsg> {
        let world = cx.world();
        match input {
            Input::Started => self.load_next(world),
            Input::Msg(RaceDirectorMsg(DirectorMsg::Loaded(result))) => self.loaded(result, world),
            Input::Msg(RaceDirectorMsg(DirectorMsg::GraceUp)) => {
                if self.restart_at.is_some() && self.phase == Phase::Racing {
                    self.move_on()
                } else {
                    Effects::none()
                }
            },
            input => {
                let (effects, transition) = self.round(&input, cx);
                let effects = match transition {
                    Some(transition) => effects.and(self.follow(transition, cx)),
                    None => effects,
                };
                effects.and(self.watch_finish(&input, cx))
            },
        }
    }
}
