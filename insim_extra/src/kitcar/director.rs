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
//! let director = RaceDirector::new(Cycle::new([fe1x, bl1])).every(Every::Races(3));
//! host.start_director(director).await?;
//! let _game = host.start(MyGame::new()).await?;
//! ```
//!
//! Pass one [`Combo`] to keep it forever, or a [`Cycle`] to loop through
//! several, in order or reshuffled each pass. For votes or layouts from disk,
//! pass a closure; see [`Rotation`].

use std::{
    fmt,
    future::Future,
    time::{Duration, Instant},
};

use futures::{
    FutureExt,
    future::{self, BoxFuture},
};
use insim::{
    core::{object::ObjectInfo, track::Track, vehicle::Vehicle},
    identifiers::ConnectionId,
    insim::{Axm, Mal, Mst, Plc, PlcAllowedCarsSet, PmoAction, PmoFlags, RaceLaps},
};

use crate::{
    kitcar::{Context, Effects, Input},
    time::TimeOfDay,
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
    /// With [`Every::Races`], the races ended on this combo and how many it
    /// gets: `(1, 3)` while the second of three runs.
    pub races: Option<(u32, u32)>,
    /// When the finished race restarts (or returns to the lobby), while the
    /// auto-restart grace period runs.
    pub restart_at: Option<Instant>,
    /// With [`Every::Duration`], when the next combo is due.
    pub rotates_at: Option<Instant>,
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

/// The layout a [`Combo`] loads.
#[derive(Debug, Clone)]
pub enum Layout {
    /// A layout saved on the server, loaded with `/axload`.
    Axload(String),
    /// Objects sent over InSim, such as from a `.lyt` file.
    Objects(Vec<ObjectInfo>),
}

/// How the grid is ordered at the start of a race (`/start`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StartOrder {
    /// The order drivers joined in.
    Fixed,
    /// The last race's finishing order.
    #[default]
    Finish,
    /// The last race's finishing order, reversed.
    Reverse,
    /// Shuffled.
    Random,
}

impl StartOrder {
    /// The host command that sets it, such as `/start finish`.
    pub fn to_command(self) -> String {
        let order = match self {
            Self::Fixed => "fixed",
            Self::Finish => "finish",
            Self::Reverse => "reverse",
            Self::Random => "random",
        };
        format!("/start {order}")
    }
}

/// How the server is set up before each race.
#[derive(Debug, Clone)]
pub struct Combo {
    /// The track. Only changed if not already loaded.
    pub track: Track,
    /// The layout, or `None` to clear it.
    pub layout: Option<Layout>,
    /// Race length.
    pub laps: RaceLaps,
    /// Wind strength (0 for none).
    pub wind: u8,
    /// The grid order. Always sent, as LFS keeps it across track changes.
    pub start: StartOrder,
    /// Whether drivers must make a pit stop (`/mustpit`). Ignored where there's
    /// no room for one: untimed, practice, or a single lap. See
    /// [`pit_stop_required`](Self::pit_stop_required). Always sent, as LFS
    /// keeps it across track changes.
    pub must_pit: bool,
    /// Vehicles allowed for everyone; empty allows them all, mods included.
    /// Mods follow the list: listed mods only, or none (`/allowmods no`) if it
    /// names only standard cars. Always sent, as LFS keeps the last
    /// restriction across track changes. To narrow it per
    /// player, answer LFS's join requests (`IsiFlags::REQ_JOIN`) in a service.
    pub vehicles: Vec<Vehicle>,
    /// The time of day: one of these, picked at random each time this combo
    /// comes up. Empty leaves the server's clock alone; it keeps its settings
    /// across track changes. Repeat one to make it likelier.
    pub time: Vec<TimeOfDay>,
}

impl Combo {
    /// An untimed session on `track`: no wind, no layout, every car allowed,
    /// the grid in finishing order, no pit stop required, and the time of day
    /// untouched.
    pub fn untimed(track: Track) -> Self {
        Self {
            track,
            layout: None,
            laps: RaceLaps::Untimed,
            wind: 0,
            start: StartOrder::default(),
            must_pit: false,
            vehicles: Vec::new(),
            time: Vec::new(),
        }
    }
    /// Always this time of day.
    pub fn time(self, time: TimeOfDay) -> Self {
        self.times([time])
    }
    /// One of these times of day, at random each time this combo comes up.
    pub fn times(mut self, times: impl IntoIterator<Item = TimeOfDay>) -> Self {
        self.time = times.into_iter().collect();
        self
    }
    /// Load this layout.
    pub fn layout(mut self, layout: Option<Layout>) -> Self {
        self.layout = layout;
        self
    }
    /// Allow only these vehicles, for everyone. Empty allows all.
    pub fn vehicles(mut self, vehicles: Vec<Vehicle>) -> Self {
        self.vehicles = vehicles;
        self
    }
    /// The grid order.
    pub fn start(mut self, start: StartOrder) -> Self {
        self.start = start;
        self
    }
    /// Require a pit stop, or not.
    pub fn must_pit(mut self, must_pit: bool) -> Self {
        self.must_pit = must_pit;
        self
    }
    /// Whether a pit stop is actually required: only if asked for, and only in
    /// a race of two or more laps, or one timed in hours.
    pub fn pit_stop_required(&self) -> bool {
        self.must_pit
            && match self.laps {
                RaceLaps::Laps(laps) => laps > 1,
                RaceLaps::Hours(_) => true,
                // Untimed, practice, and anything newer: no race to pit in.
                _ => false,
            }
    }
    /// Race length.
    pub fn laps(mut self, laps: RaceLaps) -> Self {
        self.laps = laps;
        self
    }
}

/// When the [`RaceDirector`] moves on to the next combo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Every {
    /// After this many races. A race counts when its session ends (finished,
    /// `/end` or `/restart`), not when everyone leaves.
    Races(u32),
    /// After this long on one combo, from its first race. Ends the running
    /// race after a minute's warning.
    Duration(Duration),
}

/// Why a [`Rotation`] could not produce a combo.
pub type RotationError = Box<dyn std::error::Error + Send + Sync>;

/// Picks the next combo: once at start, then each time the [`RaceDirector`]
/// moves on.
///
/// `next` runs inside the director's update, so it must not do I/O. Read what
/// you need from `current` and `world`, then do slow work (loading a `.lyt`,
/// querying a database) in the returned future. If the future fails, the
/// director stays on the current combo.
///
/// Implemented for:
/// - closures `FnMut(Option<&Combo>, &World) -> impl Future<Output =
///   Result<Combo, RotationError>>`;
/// - [`Cycle`], a list in order or reshuffled each pass;
/// - [`Combo`], which always returns itself.
///
/// ```ignore
/// RaceDirector::new(|_: Option<&Combo>, world: &World| {
///     let busy = world.connection_count() > 10;
///     async move {
///         let path = if busy { "big.lyt" } else { "small.lyt" };
///         let lyt = insim_lyt::Lyt::from_path(path)?;
///         let combo = Combo::untimed("FE1X".parse()?).layout(Some(Layout::Objects(lyt.objects)));
///         // Rust can't infer an async block's error type, so name it once.
///         Ok::<_, RotationError>(combo)
///     }
/// })
/// ```
pub trait Rotation: Send + 'static {
    /// The next combo. `current` is `None` when the director starts.
    fn next(
        &mut self,
        current: Option<&Combo>,
        world: &World,
    ) -> BoxFuture<'static, Result<Combo, RotationError>>;

    /// The combo `next` will produce, if known without asking. Shown to
    /// players as what's coming up. `None` by default, as closures can't tell.
    fn peek(&self) -> Option<&Combo> {
        None
    }
}

impl<F, Fut> Rotation for F
where
    F: FnMut(Option<&Combo>, &World) -> Fut + Send + 'static,
    Fut: Future<Output = Result<Combo, RotationError>> + Send + 'static,
{
    fn next(
        &mut self,
        current: Option<&Combo>,
        world: &World,
    ) -> BoxFuture<'static, Result<Combo, RotationError>> {
        self(current, world).boxed()
    }
}

/// A rotation that never changes.
impl Rotation for Combo {
    fn next(
        &mut self,
        _: Option<&Combo>,
        _: &World,
    ) -> BoxFuture<'static, Result<Combo, RotationError>> {
        future::ready(Ok(self.clone())).boxed()
    }

    fn peek(&self) -> Option<&Combo> {
        Some(self)
    }
}

/// A list of combos, looping after the last.
///
/// The first pass runs in the order given. With [`shuffle`](Self::shuffle),
/// each later pass is a fresh random order that never opens with the combo
/// that just played. The order is drawn as a pass ends, so
/// [`RaceDirectorStatus::next`] always names the real next combo.
#[derive(Debug, Clone)]
pub struct Cycle {
    combos: Vec<Combo>,
    /// Indices into `combos`, in this pass's order.
    order: Vec<usize>,
    /// Position in `order`.
    next: usize,
    shuffle: Option<fastrand::Rng>,
}

impl Cycle {
    /// Cycle through `combos`, in order.
    ///
    /// # Panics
    ///
    /// If `combos` is empty.
    pub fn new(combos: impl IntoIterator<Item = Combo>) -> Self {
        let combos: Vec<Combo> = combos.into_iter().collect();
        assert!(!combos.is_empty(), "a cycle needs at least one combo");
        Self {
            order: (0..combos.len()).collect(),
            combos,
            next: 0,
            shuffle: None,
        }
    }

    /// Reshuffle at the end of each pass.
    pub fn shuffle(self) -> Self {
        self.shuffle_with(fastrand::Rng::new())
    }

    /// Like [`shuffle`](Self::shuffle), from a seed, for repeatable orders.
    pub fn shuffle_seeded(self, seed: u64) -> Self {
        self.shuffle_with(fastrand::Rng::with_seed(seed))
    }

    fn shuffle_with(mut self, rng: fastrand::Rng) -> Self {
        self.shuffle = Some(rng);
        self
    }

    /// Start the next pass, reshuffled if asked, never opening with `last`.
    fn next_pass(&mut self, last: usize) {
        self.next = 0;
        let Some(rng) = &mut self.shuffle else {
            return;
        };
        rng.shuffle(&mut self.order);
        if self.order.len() > 1 && self.order[0] == last {
            let swap = rng.usize(1..self.order.len());
            self.order.swap(0, swap);
        }
    }
}

impl Rotation for Cycle {
    fn next(
        &mut self,
        _: Option<&Combo>,
        _: &World,
    ) -> BoxFuture<'static, Result<Combo, RotationError>> {
        let index = self.order[self.next];
        self.next += 1;
        if self.next == self.order.len() {
            self.next_pass(index);
        }
        future::ready(Ok(self.combos[index].clone())).boxed()
    }

    fn peek(&self) -> Option<&Combo> {
        self.combos.get(self.order[self.next])
    }
}

/// Steers the [`RaceDirector`] from a service, with
/// [`Effects::director`](crate::kitcar::Effects::director).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DirectorCommand {
    /// Move on to the next combo now, ending the running race. Ignored while
    /// the next combo is already loading.
    Rotate,
}

/// The [`RaceDirector`]'s own messages: its timers, and combos as they load.
#[derive(Debug, Clone)]
pub(crate) struct RaceDirectorMsg(DirectorMsg);

#[derive(Debug, Clone)]
enum DirectorMsg {
    /// A minute left on this combo.
    Warning,
    /// Time is up on this combo.
    TimeUp,
    /// The rotation produced the next combo, or failed to.
    Loaded(Result<Combo, String>),
    /// The grace period after the first finisher is over.
    GraceUp,
}

#[derive(Debug, PartialEq)]
enum RotationTimer {
    Warning,
    TimeUp,
}

/// Kept apart from [`RotationTimer`], which a new combo cancels.
#[derive(Debug, PartialEq)]
struct Grace;

/// How long before a timed rotation players are warned.
const WARNING: Duration = Duration::from_secs(60);

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
    every: Option<Every>,
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
    /// Races ended on the current combo, for [`Every::Races`].
    races: u32,
    /// Whether the [`Every::Duration`] timer is running for the current combo.
    timed: bool,
    /// When that timer moves on.
    rotates_at: Option<Instant>,
    /// The grace period after the first finisher, if auto-restarting.
    auto_restart: Option<Duration>,
    /// Someone has finished, and the grace period is running.
    finishing: bool,
    /// When the grace period ends, while `finishing`.
    restart_at: Option<Instant>,
    /// `/restart` or `/end` is sent; ignore finishes until the session ends.
    moved_on: bool,
    /// Picks each combo's time of day.
    rng: fastrand::Rng,
}

impl fmt::Debug for RaceDirector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RaceDirector")
            .field("every", &self.every)
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
    /// closure. Stays on the first combo unless you set
    /// [`every`](Self::every).
    pub fn new(rotation: impl Rotation) -> Self {
        Self {
            rotation: Box::new(rotation),
            every: None,
            current: None,
            set_up: None,
            loading: false,
            phase: Phase::Waiting,
            configured: false,
            ending_session: false,
            races: 0,
            timed: false,
            rotates_at: None,
            auto_restart: None,
            finishing: false,
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

    /// Move on to the next combo this often.
    pub fn every(mut self, every: Every) -> Self {
        self.every = Some(every);
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
            restart_at: self.restart_at(),
            rotates_at: self.rotates_at,
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

    /// With [`Every::Races`], the races ended on this combo and how many it
    /// gets: `(1, 3)` while the second of three runs.
    pub(crate) fn races(&self) -> Option<(u32, u32)> {
        match self.every {
            Some(Every::Races(races)) => Some((self.races, races)),
            _ => None,
        }
    }

    /// When the finished race restarts (or returns to the lobby), while
    /// [`auto_restart`](Self::auto_restart)'s grace period runs.
    pub(crate) fn restart_at(&self) -> Option<Instant> {
        self.restart_at.filter(|_| self.finishing)
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
        Effects::new().task(async move {
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
        self.timed = false;
        self.rotates_at = None;
        let effects = Effects::new().cancel_all::<RotationTimer>();
        let effects = match result {
            Ok(combo) => {
                let first = self.current.is_none();
                let track = combo.track;
                tracing::info!(%track, "combo loaded");
                self.current = Some(combo);
                self.configured = false;
                if first {
                    effects
                } else {
                    effects.message(format!("^3Next track: ^7{track}"), ConnectionId::ALL)
                }
            },
            Err(err) => {
                tracing::warn!(error = %err, "rotation failed to produce a combo");
                match &self.current {
                    Some(combo) => effects.message(
                        format!(
                            "^1Couldn't load the next track - staying on ^7{}",
                            combo.track
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

    /// Count races and start the clock, for the rotation.
    fn follow(&mut self, transition: Transition, cx: &Context<'_>) -> Effects<RaceDirectorMsg> {
        let world = cx.world();
        match (transition, self.every) {
            (Transition::Racing, Some(Every::Duration(after))) if !self.timed => {
                self.timed = true;
                self.rotates_at = Some(cx.now() + after);
                let effects = Effects::new().schedule_after(
                    RotationTimer::TimeUp,
                    after,
                    RaceDirectorMsg(DirectorMsg::TimeUp),
                );
                match after.checked_sub(WARNING) {
                    Some(warn) if !warn.is_zero() => effects.schedule_after(
                        RotationTimer::Warning,
                        warn,
                        RaceDirectorMsg(DirectorMsg::Warning),
                    ),
                    _ => effects,
                }
            },
            (Transition::Ended { shortfall: false }, Some(Every::Races(races))) => {
                self.races += 1;
                if self.races >= races && !self.loading {
                    self.load_next(world)
                } else {
                    Effects::none()
                }
            },
            _ => Effects::none(),
        }
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
            self.finishing = false;
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
        let first = finished && !self.finishing;
        if first {
            tracing::info!(?grace, "first finisher, grace period started");
            self.finishing = true;
            self.restart_at = Some(cx.now() + grace);
        }
        let Some(restart_at) = self.restart_at.filter(|_| self.finishing) else {
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
        self.finishing = false;
        self.restart_at = None;
        self.moved_on = true;
        // The race being ended counts towards `Every::Races`.
        let due = matches!(self.every, Some(Every::Races(races)) if self.races + 1 >= races);
        let next = if due || self.loading || !self.configured {
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
        }
    }

    /// Move on, ending the race if one is running.
    fn time_up(&mut self, world: &World) -> Effects<RaceDirectorMsg> {
        if self.loading {
            return Effects::none();
        }
        tracing::info!("time is up on this combo");
        self.rotate_now(world)
    }

    /// Load the next combo, ending the race if one is running.
    fn rotate_now(&mut self, world: &World) -> Effects<RaceDirectorMsg> {
        let effects = self.load_next(world);
        match self.phase {
            // The lobby that follows waits for the next combo.
            Phase::Racing => effects.packet(Mst::new("/end")),
            Phase::SettingUp | Phase::Waiting => effects,
        }
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

    /// Set up the lobby, or just prompt for `/ready` if it's done. While a
    /// combo loads, ask players to wait.
    fn set_up<M: Send + 'static>(&mut self, world: &World) -> Effects<M> {
        let combo = match &self.current {
            Some(combo) if !self.loading => combo,
            _ => return Effects::new().message(LOADING, ConnectionId::ALL),
        };
        if self.configured {
            return Effects::new().message(READY, ConnectionId::ALL);
        }
        let time = (!combo.time.is_empty()).then(|| combo.time[self.rng.usize(..combo.time.len())]);
        let (effects, done) = self.configure(combo, time, world);
        self.configured = done;
        if done {
            tracing::info!(track = %combo.track, "lobby set up, waiting for /ready");
            self.set_up = Some(combo.clone());
            effects.message(READY, ConnectionId::ALL)
        } else {
            effects
        }
    }

    /// Returns the packets and whether setup is complete. LFS changes track
    /// asynchronously, so nothing else is sent until `TrackChanged`.
    fn configure<M: Send + 'static>(
        &self,
        combo: &Combo,
        time: Option<TimeOfDay>,
        world: &World,
    ) -> (Effects<M>, bool) {
        if world.track() != Some(combo.track) {
            tracing::info!(track = %combo.track, "changing track");
            return (
                Effects::new()
                    .packet(Mst::new("/clear"))
                    .packet(Mst::new(format!("/track {}", combo.track))),
                false,
            );
        }
        if let Some(time) = time {
            tracing::info!(track = %combo.track, ?time, "time of day");
        }
        let effects = Effects::new()
            .packet(Mst::new(combo.laps.to_command()))
            .packet(Mst::new(format!("/wind {}", combo.wind)))
            .packet(Mst::new(combo.start.to_command()))
            .packet(Mst::new(if combo.pit_stop_required() {
                "/mustpit yes"
            } else {
                "/mustpit no"
            }))
            .packets(time.iter().flat_map(TimeOfDay::to_commands).map(Mst::new))
            .packets(restrict_everyone(&combo.vehicles))
            .packet(Mst::new("/axclear"));
        let effects = match &combo.layout {
            None => effects,
            Some(Layout::Axload(name)) => effects
                .message(
                    "^3Setting up - loading layout, please wait...",
                    ConnectionId::ALL,
                )
                .packet(Mst::new(format!("/axload {name}"))),
            Some(Layout::Objects(objects)) => {
                let chunks = objects.chunks(60);
                let last = chunks.len().saturating_sub(1);
                effects.packets(chunks.enumerate().map(|(index, chunk)| Axm {
                    ucid: ConnectionId::LOCAL,
                    flags: if index == last {
                        PmoFlags::FILE_END
                    } else {
                        PmoFlags::empty()
                    },
                    action: PmoAction::AddObjects(chunk.to_vec()),
                    ..Default::default()
                }))
            },
        };
        (effects, true)
    }
}

/// Restrict everyone to `vehicles`, or allow everything if empty. Standard
/// cars go in a `Plc` to every connection, mods in a server-wide `Mal`. An
/// empty `Mal` allows any mod, so a list of only standard cars turns mods off
/// with `/allowmods no` instead. Every `Mal` goes with `/unapproved yes`: the
/// list is the authority, whether or not LFS has approved the mods on it.
fn restrict_everyone(vehicles: &[Vehicle]) -> Vec<insim::Packet> {
    if vehicles.is_empty() {
        return vec![
            Mst::new("/allowmods yes").into(),
            Mst::new("/unapproved yes").into(),
            Plc {
                ucid: ConnectionId::ALL,
                cars: PlcAllowedCarsSet::all(),
                ..Default::default()
            }
            .into(),
            Mal::default().into(),
        ];
    }
    let mut cars = PlcAllowedCarsSet::default();
    let mut mods = Mal::default();
    for vehicle in vehicles {
        match vehicle {
            Vehicle::Mod(_) => {
                let _ = mods.insert(*vehicle);
            },
            _ => {
                let _ = cars.insert(*vehicle);
            },
        }
    }
    let plc = Plc {
        ucid: ConnectionId::ALL,
        cars,
        ..Default::default()
    };
    if mods.is_empty() {
        vec![Mst::new("/allowmods no").into(), plc.into()]
    } else {
        vec![
            Mst::new("/allowmods yes").into(),
            Mst::new("/unapproved yes").into(),
            plc.into(),
            mods.into(),
        ]
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
            Input::Msg(RaceDirectorMsg(DirectorMsg::Warning)) => {
                Effects::new().message("^3Track changes in 1 minute", ConnectionId::ALL)
            },
            Input::Msg(RaceDirectorMsg(DirectorMsg::TimeUp)) => self.time_up(world),
            Input::Msg(RaceDirectorMsg(DirectorMsg::Loaded(result))) => self.loaded(result, world),
            Input::Msg(RaceDirectorMsg(DirectorMsg::GraceUp)) => {
                if self.finishing && self.phase == Phase::Racing {
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

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use insim::{
        Packet,
        identifiers::PlayerId,
        insim::{Cnl, Fin, Mal, Mst, Ncn, Npl, Plc, Pll, RaceInProgress, Rst, Sta},
    };

    use super::*;

    /// A director whose combo has already loaded, to test the round alone.
    fn loaded(combo: Combo) -> RaceDirector {
        let mut director = RaceDirector::new(combo.clone());
        director.current = Some(combo);
        director
    }

    /// Apply a packet to the world, then pass it and its events on, as the host
    /// does. Returns the commands sent and transitions reported.
    fn feed(
        world: &mut World,
        director: &mut RaceDirector,
        packet: Packet,
    ) -> (Vec<String>, Vec<Transition>) {
        let events = world.apply(&packet);
        let world = &*world;
        let cx = Context::new(world, Instant::now());
        let mut commands = Vec::new();
        let mut transitions = Vec::new();
        let inputs = std::iter::once(Input::<()>::Packet(packet))
            .chain(events.into_iter().map(Input::World));
        for input in inputs {
            let (effects, transition) = director.round(&input, &cx);
            commands.extend(effects.packets_sent().filter_map(|p| match p {
                Packet::Mst(Mst { msg, .. }) => Some(msg.clone()),
                _ => None,
            }));
            transitions.extend(transition);
        }
        (commands, transitions)
    }

    #[test]
    fn sets_up_in_the_lobby_then_races_and_ends_when_everyone_leaves() {
        let track: Track = "FE1X".parse().unwrap();
        let mut world = World::new();
        let mut director = loaded(Combo::untimed(track));

        // The lobby, already on the right track.
        let _ = feed(
            &mut world,
            &mut director,
            Sta {
                track,
                raceinprog: RaceInProgress::No,
                ..Default::default()
            }
            .into(),
        );
        let (commands, transitions) = feed(
            &mut world,
            &mut director,
            Ncn {
                ucid: ConnectionId(1),
                uname: "driver".into(),
                ..Default::default()
            }
            .into(),
        );
        assert_eq!(transitions, [Transition::SetupStarted]);
        assert!(commands.iter().any(|c| c == "/axclear"), "{commands:?}");
        assert!(!commands.iter().any(|c| c.starts_with("/track")));
        assert_eq!(director.phase(), Phase::SettingUp);

        // /ready starts the session.
        let (_, transitions) = feed(
            &mut world,
            &mut director,
            Rst {
                track,
                ..Default::default()
            }
            .into(),
        );
        assert_eq!(transitions, [Transition::Racing]);
        assert!(director.racing());

        // Everyone leaves mid-race: end it.
        let (commands, transitions) = feed(
            &mut world,
            &mut director,
            Cnl {
                ucid: ConnectionId(1),
                ..Default::default()
            }
            .into(),
        );
        assert_eq!(transitions, [Transition::Ended { shortfall: true }]);
        assert_eq!(commands, ["/end"]);
        assert_eq!(director.phase(), Phase::Waiting);
        // A rotation picks the next track; once LFS is back in the lobby (after
        // the /end), the next player to join triggers the change.
        director.current = Some(Combo::untimed("BL1".parse().unwrap()));
        director.configured = false;
        let _ = feed(
            &mut world,
            &mut director,
            Sta {
                track,
                raceinprog: RaceInProgress::No,
                ..Default::default()
            }
            .into(),
        );
        let (commands, _) = feed(
            &mut world,
            &mut director,
            Ncn {
                ucid: ConnectionId(2),
                ..Default::default()
            }
            .into(),
        );
        assert_eq!(commands, ["/clear", "/track BL1"]);
    }

    #[test]
    fn changes_track_first_and_waits_for_it_to_load() {
        let mut world = World::new();
        let mut director = loaded(Combo::untimed("BL1".parse().unwrap()));
        let _ = feed(
            &mut world,
            &mut director,
            Sta {
                track: "FE1X".parse().unwrap(),
                ..Default::default()
            }
            .into(),
        );
        let (commands, transitions) = feed(
            &mut world,
            &mut director,
            Ncn {
                ucid: ConnectionId(1),
                ..Default::default()
            }
            .into(),
        );
        assert_eq!(transitions, [Transition::SetupStarted]);
        assert_eq!(commands, ["/clear", "/track BL1"]);
    }

    #[test]
    fn races_whoever_starts_the_session_and_restarts_end_one_round() {
        let track: Track = "FE1X".parse().unwrap();
        let mut world = World::new();
        let mut director = loaded(Combo::untimed(track));
        let rst = || -> Packet {
            Rst {
                track,
                ..Default::default()
            }
            .into()
        };
        let _ = feed(
            &mut world,
            &mut director,
            Sta {
                track,
                raceinprog: RaceInProgress::No,
                ..Default::default()
            }
            .into(),
        );
        let _ = feed(
            &mut world,
            &mut director,
            Ncn {
                ucid: ConnectionId(1),
                uname: "driver".into(),
                ..Default::default()
            }
            .into(),
        );
        let (_, transitions) = feed(&mut world, &mut director, rst());
        assert_eq!(transitions, [Transition::Racing]);

        // An admin's /restart mid-race: that round ends and the next one races.
        let (commands, transitions) = feed(&mut world, &mut director, rst());
        assert_eq!(
            transitions,
            [Transition::Ended { shortfall: false }, Transition::Racing]
        );
        assert!(commands.is_empty(), "{commands:?}");
        assert!(director.racing());

        // An admin's /end: back to the lobby, which is set up again.
        let (_, transitions) = feed(
            &mut world,
            &mut director,
            Sta {
                track,
                raceinprog: RaceInProgress::No,
                ..Default::default()
            }
            .into(),
        );
        assert_eq!(
            transitions,
            [
                Transition::Ended { shortfall: false },
                Transition::SetupStarted
            ]
        );
        assert_eq!(director.phase(), Phase::SettingUp);
    }

    /// What the service did, with tasks run to completion and their messages
    /// handled.
    #[derive(Debug, Default)]
    struct Seen {
        commands: Vec<String>,
        chat: Vec<String>,
        timers: usize,
    }

    /// Pass `input` in as the host would: packets hit the world first, then
    /// their events. Tasks are awaited and their messages passed back in.
    async fn serve(
        world: &mut World,
        director: &mut RaceDirector,
        input: Input<RaceDirectorMsg>,
    ) -> Seen {
        let mut queue: std::collections::VecDeque<_> = match input {
            Input::Packet(packet) => {
                let events = world.apply(&packet);
                std::iter::once(Input::Packet(packet))
                    .chain(events.into_iter().map(Input::World))
                    .collect()
            },
            input => [input].into(),
        };
        let mut seen = Seen::default();
        while let Some(input) = queue.pop_front() {
            let cx = Context::new(world, Instant::now());
            let mut effects = director.update(input, &cx);
            for task in effects.take_tasks() {
                queue.push_back(Input::Msg(task.await));
            }
            for packet in effects.packets_sent() {
                match packet {
                    Packet::Mst(Mst { msg, .. }) => seen.commands.push(msg.clone()),
                    Packet::Mtc(mtc) => seen.chat.push(mtc.text.clone()),
                    _ => {},
                }
            }
            seen.timers += effects.timers().count();
        }
        seen
    }

    fn lobby(track: Track) -> Input<RaceDirectorMsg> {
        Input::Packet(
            Sta {
                track,
                raceinprog: RaceInProgress::No,
                ..Default::default()
            }
            .into(),
        )
    }

    fn start(track: Track) -> Input<RaceDirectorMsg> {
        Input::Packet(
            Rst {
                track,
                ..Default::default()
            }
            .into(),
        )
    }

    fn join() -> Input<RaceDirectorMsg> {
        Input::Packet(
            Ncn {
                ucid: ConnectionId(1),
                uname: "driver".into(),
                ..Default::default()
            }
            .into(),
        )
    }

    fn msg(msg: DirectorMsg) -> Input<RaceDirectorMsg> {
        Input::Msg(RaceDirectorMsg(msg))
    }

    /// Started, in a lobby on `track` with one player, and set up.
    async fn ready(director: RaceDirector, track: Track) -> (World, RaceDirector) {
        let mut world = World::new();
        let mut director = director;
        let _ = serve(&mut world, &mut director, Input::Started).await;
        let _ = serve(&mut world, &mut director, lobby(track)).await;
        let _ = serve(&mut world, &mut director, join()).await;
        assert_eq!(director.phase(), Phase::SettingUp);
        (world, director)
    }

    fn fe1x() -> Track {
        "FE1X".parse().unwrap()
    }

    fn bl1() -> Track {
        "BL1".parse().unwrap()
    }

    fn cycle() -> Cycle {
        Cycle::new([Combo::untimed(fe1x()), Combo::untimed(bl1())])
    }

    #[tokio::test]
    async fn every_races_moves_on_after_that_many_races() {
        let director = RaceDirector::new(cycle()).every(Every::Races(2));
        let (mut world, mut director) = ready(director, fe1x()).await;

        let _ = serve(&mut world, &mut director, start(fe1x())).await;
        let first = serve(&mut world, &mut director, lobby(fe1x())).await;
        assert!(first.chat.iter().all(|c| !c.contains("Next track")));
        assert!(!first.commands.iter().any(|c| c.starts_with("/track")));

        let _ = serve(&mut world, &mut director, start(fe1x())).await;
        let second = serve(&mut world, &mut director, lobby(fe1x())).await;
        assert!(
            second.chat.iter().any(|c| c.contains("Next track: ^7BL1")),
            "{second:?}"
        );
        assert_eq!(second.commands, ["/clear", "/track BL1"]);
        assert_eq!(director.combo().unwrap().track, bl1());
    }

    #[tokio::test]
    async fn everyone_leaving_does_not_count_as_a_race() {
        let director = RaceDirector::new(cycle()).every(Every::Races(1));
        let (mut world, mut director) = ready(director, fe1x()).await;

        let _ = serve(&mut world, &mut director, start(fe1x())).await;
        let left = serve(
            &mut world,
            &mut director,
            Input::Packet(
                Cnl {
                    ucid: ConnectionId(1),
                    ..Default::default()
                }
                .into(),
            ),
        )
        .await;
        assert_eq!(left.commands, ["/end"]);
        assert_eq!(director.combo().unwrap().track, fe1x());
    }

    #[tokio::test]
    async fn every_duration_warns_then_ends_the_race_and_moves_on() {
        let director = RaceDirector::new(cycle()).every(Every::Duration(Duration::from_secs(600)));
        let (mut world, mut director) = ready(director, fe1x()).await;

        // The clock starts with the first race on this combo...
        let started = serve(&mut world, &mut director, start(fe1x())).await;
        assert_eq!(started.timers, 2, "time up and its warning");
        // ...and a /restart doesn't restart it.
        let restarted = serve(&mut world, &mut director, start(fe1x())).await;
        assert_eq!(restarted.timers, 0);

        let warned = serve(&mut world, &mut director, msg(DirectorMsg::Warning)).await;
        assert_eq!(warned.chat, ["^3Track changes in 1 minute"]);

        let up = serve(&mut world, &mut director, msg(DirectorMsg::TimeUp)).await;
        assert_eq!(up.commands, ["/end"]);
        assert!(up.chat.iter().any(|c| c.contains("Next track: ^7BL1")));

        // Back in the lobby, the next track is set up.
        let lobby = serve(&mut world, &mut director, lobby(fe1x())).await;
        assert_eq!(lobby.commands, ["/clear", "/track BL1"]);
    }

    #[tokio::test]
    async fn a_single_combo_never_moves_on() {
        let director = RaceDirector::new(Combo::untimed(fe1x()));
        let (mut world, mut director) = ready(director, fe1x()).await;

        for _ in 0..3 {
            let started = serve(&mut world, &mut director, start(fe1x())).await;
            assert_eq!(started.timers, 0);
            let ended = serve(&mut world, &mut director, lobby(fe1x())).await;
            assert!(ended.chat.iter().all(|c| !c.contains("Next track")));
            assert!(!ended.commands.iter().any(|c| c.starts_with("/track")));
        }
        assert_eq!(director.combo().unwrap().track, fe1x());
    }

    #[tokio::test]
    async fn a_closure_picks_each_combo_from_the_world() {
        // The first combo depends on who is online when the director starts.
        let rotation = |current: Option<&Combo>, world: &World| {
            let track = match (current, world.connection_count()) {
                (None, 0) => fe1x(),
                _ => bl1(),
            };
            async move { Ok::<_, RotationError>(Combo::untimed(track)) }
        };
        let director = RaceDirector::new(rotation).every(Every::Races(1));
        let (mut world, mut director) = ready(director, fe1x()).await;
        assert_eq!(director.combo().unwrap().track, fe1x());

        let _ = serve(&mut world, &mut director, start(fe1x())).await;
        let ended = serve(&mut world, &mut director, lobby(fe1x())).await;
        assert_eq!(ended.commands, ["/clear", "/track BL1"]);
    }

    #[tokio::test]
    async fn a_failed_rotation_stays_on_the_current_combo() {
        let mut calls = 0;
        let rotation = move |_: Option<&Combo>, _: &World| {
            calls += 1;
            let first = calls == 1;
            async move {
                if first {
                    Ok(Combo::untimed(fe1x()))
                } else {
                    Err(RotationError::from("layout file is missing"))
                }
            }
        };
        let director = RaceDirector::new(rotation).every(Every::Races(1));
        let (mut world, mut director) = ready(director, fe1x()).await;

        let _ = serve(&mut world, &mut director, start(fe1x())).await;
        let ended = serve(&mut world, &mut director, lobby(fe1x())).await;
        assert!(
            ended
                .chat
                .iter()
                .any(|c| c.contains("Couldn't load the next track - staying on ^7FE1X")),
            "{ended:?}"
        );
        assert!(!ended.commands.iter().any(|c| c.starts_with("/track")));
        assert_eq!(director.combo().unwrap().track, fe1x());
        assert_eq!(director.phase(), Phase::SettingUp);
    }

    #[tokio::test]
    async fn the_lobby_waits_for_a_combo_still_loading() {
        let mut world = World::new();
        let mut director = RaceDirector::new(Combo::untimed(bl1()));
        // Start, but hold on to the load instead of finishing it.
        let cx = Context::new(&world, Instant::now());
        let mut started = director.update(Input::Started, &cx);
        let load = started.take_tasks().pop().expect("the first combo loads");

        let _ = serve(&mut world, &mut director, lobby(fe1x())).await;
        let waiting = serve(&mut world, &mut director, join()).await;
        assert!(waiting.commands.is_empty(), "{waiting:?}");
        assert!(
            waiting
                .chat
                .iter()
                .any(|c| c.contains("Loading the next track"))
        );

        // Once it arrives, the lobby is set up.
        let arrived = serve(&mut world, &mut director, Input::Msg(load.await)).await;
        assert_eq!(arrived.commands, ["/clear", "/track BL1"]);
    }

    /// A 5-lap race on FE1X with players 1 and 2 on track.
    async fn two_racing(director: RaceDirector) -> (World, RaceDirector) {
        let (mut world, mut director) = ready(director, fe1x()).await;
        let _ = serve(
            &mut world,
            &mut director,
            Input::Packet(
                Ncn {
                    ucid: ConnectionId(2),
                    uname: "other".into(),
                    ..Default::default()
                }
                .into(),
            ),
        )
        .await;
        // A lap race: untimed sessions have no finish.
        let race = Input::Packet(
            Rst {
                track: fe1x(),
                racelaps: RaceLaps::Laps(5),
                ..Default::default()
            }
            .into(),
        );
        let _ = serve(&mut world, &mut director, race).await;
        for (ucid, plid) in [(1, 1), (2, 2)] {
            let _ = serve(
                &mut world,
                &mut director,
                Input::Packet(
                    Npl {
                        ucid: ConnectionId(ucid),
                        plid: PlayerId(plid),
                        nump: 1,
                        ..Default::default()
                    }
                    .into(),
                ),
            )
            .await;
        }
        assert_eq!(world.live_entrants().count(), 2);
        (world, director)
    }

    fn fin(plid: u8) -> Input<RaceDirectorMsg> {
        Input::Packet(
            Fin {
                plid: PlayerId(plid),
                ..Default::default()
            }
            .into(),
        )
    }

    #[tokio::test]
    async fn auto_restart_restarts_once_the_field_is_home() {
        let director =
            RaceDirector::new(Combo::untimed(fe1x())).auto_restart(Duration::from_secs(60));
        let (mut world, mut director) = two_racing(director).await;

        let first = serve(&mut world, &mut director, fin(1)).await;
        assert_eq!(first.timers, 1, "the grace period");
        assert_eq!(first.chat, ["^3Next race in 60s"]);
        assert!(first.commands.is_empty());

        // Everyone's home: the wait is cut short, and players are told.
        let second = serve(&mut world, &mut director, fin(2)).await;
        assert_eq!(second.chat, ["^3All drivers finished - next race in 10s"]);
        assert_eq!(second.timers, 1, "the shortened grace period");
        assert!(second.commands.is_empty());
        let up = serve(&mut world, &mut director, msg(DirectorMsg::GraceUp)).await;
        assert_eq!(up.commands, ["/restart"]);

        // LFS restarts the race; its first finisher starts a new grace period.
        let restarted = Input::Packet(
            Rst {
                track: fe1x(),
                racelaps: RaceLaps::Laps(5),
                ..Default::default()
            }
            .into(),
        );
        let _ = serve(&mut world, &mut director, restarted).await;
        // Drivers rejoin the grid.
        for (ucid, plid) in [(1, 1), (2, 2)] {
            let _ = serve(
                &mut world,
                &mut director,
                Input::Packet(
                    Npl {
                        ucid: ConnectionId(ucid),
                        plid: PlayerId(plid),
                        nump: 1,
                        ..Default::default()
                    }
                    .into(),
                ),
            )
            .await;
        }
        let next = serve(&mut world, &mut director, fin(1)).await;
        assert_eq!(next.chat, ["^3Next race in 60s"]);
    }

    #[tokio::test]
    async fn auto_restart_stops_waiting_when_the_grace_period_ends() {
        let director =
            RaceDirector::new(Combo::untimed(fe1x())).auto_restart(Duration::from_secs(60));
        let (mut world, mut director) = two_racing(director).await;

        let _ = serve(&mut world, &mut director, fin(1)).await;
        let up = serve(&mut world, &mut director, msg(DirectorMsg::GraceUp)).await;
        assert_eq!(up.commands, ["/restart"]);
    }

    #[tokio::test]
    async fn auto_restart_counts_drivers_who_leave_as_home() {
        let director =
            RaceDirector::new(Combo::untimed(fe1x())).auto_restart(Duration::from_secs(60));
        let (mut world, mut director) = two_racing(director).await;

        let _ = serve(&mut world, &mut director, fin(1)).await;
        let left = serve(
            &mut world,
            &mut director,
            Input::Packet(
                Pll {
                    plid: PlayerId(2),
                    ..Default::default()
                }
                .into(),
            ),
        )
        .await;
        assert_eq!(left.chat, ["^3All drivers finished - next race in 10s"]);
        assert!(left.commands.is_empty());
        let up = serve(&mut world, &mut director, msg(DirectorMsg::GraceUp)).await;
        assert_eq!(up.commands, ["/restart"]);
    }

    /// Player 1 alone on track, in a 5-lap race on FE1X.
    async fn solo_racing(director: RaceDirector) -> (World, RaceDirector) {
        let (mut world, mut director) = ready(director, fe1x()).await;
        let race = Input::Packet(
            Rst {
                track: fe1x(),
                racelaps: RaceLaps::Laps(5),
                ..Default::default()
            }
            .into(),
        );
        let _ = serve(&mut world, &mut director, race).await;
        let joined = Input::Packet(
            Npl {
                ucid: ConnectionId(1),
                plid: PlayerId(1),
                nump: 1,
                ..Default::default()
            }
            .into(),
        );
        let _ = serve(&mut world, &mut director, joined).await;
        (world, director)
    }

    #[tokio::test]
    async fn auto_restart_alone_waits_briefly_and_says_so() {
        let director =
            RaceDirector::new(Combo::untimed(fe1x())).auto_restart(Duration::from_secs(60));
        let (mut world, mut director) = solo_racing(director).await;

        // One honest announcement, not "60s" followed by an instant restart.
        let home = serve(&mut world, &mut director, fin(1)).await;
        assert_eq!(home.chat, ["^3All drivers finished - next race in 10s"]);
        assert_eq!(home.timers, 1);
        assert!(home.commands.is_empty(), "{home:?}");

        let up = serve(&mut world, &mut director, msg(DirectorMsg::GraceUp)).await;
        assert_eq!(up.commands, ["/restart"]);
    }

    #[tokio::test]
    async fn auto_restart_keeps_a_grace_period_shorter_than_the_settle() {
        let director =
            RaceDirector::new(Combo::untimed(fe1x())).auto_restart(Duration::from_secs(5));
        let (mut world, mut director) = solo_racing(director).await;

        let home = serve(&mut world, &mut director, fin(1)).await;
        assert_eq!(home.chat, ["^3Next race in 5s"]);
        assert!(home.commands.is_empty());
    }

    #[tokio::test]
    async fn auto_restart_returns_to_the_lobby_when_the_next_combo_is_due() {
        let director = RaceDirector::new(cycle())
            .every(Every::Races(1))
            .auto_restart(Duration::from_secs(60));
        let (mut world, mut director) = two_racing(director).await;

        let _ = serve(&mut world, &mut director, fin(1)).await;
        let _ = serve(&mut world, &mut director, fin(2)).await;
        let up = serve(&mut world, &mut director, msg(DirectorMsg::GraceUp)).await;
        assert_eq!(up.commands, ["/end"]);

        // The lobby that follows sets the next combo up.
        let lobby = serve(&mut world, &mut director, lobby(fe1x())).await;
        assert_eq!(lobby.commands, ["/clear", "/track BL1"]);
    }

    #[tokio::test]
    async fn without_auto_restart_finished_races_are_left_alone() {
        let director = RaceDirector::new(Combo::untimed(fe1x()));
        let (mut world, mut director) = two_racing(director).await;

        for plid in [1, 2] {
            let seen = serve(&mut world, &mut director, fin(plid)).await;
            assert!(seen.commands.is_empty() && seen.chat.is_empty() && seen.timers == 0);
        }
    }

    #[tokio::test]
    async fn a_late_grace_period_does_nothing_once_the_race_is_over() {
        let director =
            RaceDirector::new(Combo::untimed(fe1x())).auto_restart(Duration::from_secs(60));
        let (mut world, mut director) = two_racing(director).await;

        let _ = serve(&mut world, &mut director, fin(1)).await;
        // An admin ends it first.
        let _ = serve(&mut world, &mut director, lobby(fe1x())).await;
        let up = serve(&mut world, &mut director, msg(DirectorMsg::GraceUp)).await;
        assert!(up.commands.is_empty(), "{up:?}");
    }

    /// The commands sent setting up a lobby already on `combo`'s track.
    fn set_up_commands(combo: Combo) -> Vec<String> {
        set_up_commands_with(RaceDirector::new(combo.clone()), combo)
    }

    /// Like [`set_up_commands`], with `director` already configured.
    fn set_up_commands_with(mut director: RaceDirector, combo: Combo) -> Vec<String> {
        let track = combo.track;
        let mut world = World::new();
        director.current = Some(combo);
        let lobby = Sta {
            track,
            raceinprog: RaceInProgress::No,
            ..Default::default()
        };
        let _ = feed(&mut world, &mut director, lobby.into());
        let joined = Ncn {
            ucid: ConnectionId(1),
            uname: "driver".into(),
            ..Default::default()
        };
        feed(&mut world, &mut director, joined.into()).0
    }

    #[test]
    fn sets_the_time_of_day_with_the_lobby() {
        use crate::time::TimeOfDay;

        let commands = set_up_commands(Combo::untimed(fe1x()).time(TimeOfDay::fixed(17, 30)));
        let wind = commands.iter().position(|c| c.starts_with("/wind"));
        let set = commands.iter().position(|c| c == "/time set 17:30");
        let mul = commands.iter().position(|c| c == "/time mul 0");
        assert!(wind < set && set < mul && wind.is_some(), "{commands:?}");

        // Without one, the server's clock is left alone.
        let untouched = set_up_commands(Combo::untimed(fe1x()));
        assert!(
            !untouched.iter().any(|c| c.starts_with("/time")),
            "{untouched:?}"
        );
    }

    /// The tracks `cycle` produces over `count` combos, checking each against
    /// what it said was next.
    fn drawn(cycle: &mut Cycle, count: usize) -> Vec<Track> {
        let world = World::new();
        (0..count)
            .map(|_| {
                let peeked = cycle.peek().map(|combo| combo.track);
                let combo = cycle
                    .next(None, &world)
                    .now_or_never()
                    .expect("a cycle is ready at once")
                    .expect("a cycle never fails");
                assert_eq!(peeked, Some(combo.track), "peek named the next combo");
                combo.track
            })
            .collect()
    }

    fn as1() -> Track {
        "AS1".parse().unwrap()
    }

    #[test]
    fn a_shuffled_cycle_plays_each_once_per_pass_and_never_twice_running() {
        let tracks = [fe1x(), bl1(), as1()];
        for seed in 0..50 {
            let mut cycle = Cycle::new(tracks.map(Combo::untimed)).shuffle_seeded(seed);
            let drawn = drawn(&mut cycle, tracks.len() * 10);

            // The first pass is in the order given.
            assert_eq!(drawn[..3], tracks, "seed {seed}");
            for pass in drawn.chunks(tracks.len()) {
                let mut sorted = pass.to_vec();
                sorted.sort_by_key(ToString::to_string);
                let mut all = tracks.to_vec();
                all.sort_by_key(ToString::to_string);
                assert_eq!(sorted, all, "seed {seed}: every track once a pass");
            }
            assert!(
                drawn.windows(2).all(|pair| pair[0] != pair[1]),
                "seed {seed}: {drawn:?}"
            );
        }
    }

    #[test]
    fn a_cycle_of_one_repeats_it_even_shuffled() {
        let mut cycle = Cycle::new([Combo::untimed(fe1x())]).shuffle_seeded(7);
        assert_eq!(drawn(&mut cycle, 3), [fe1x(); 3]);
    }

    #[test]
    fn picks_one_of_several_times_each_time_a_combo_comes_up() {
        use crate::time::TimeOfDay;

        let night = "/time set 00:00";
        let noon = "/time set 12:00";
        let mut seen = std::collections::HashSet::new();
        for seed in 0..20 {
            let combo =
                Combo::untimed(fe1x()).times([TimeOfDay::fixed(0, 0), TimeOfDay::fixed(12, 0)]);
            let commands = set_up_commands_with(RaceDirector::new(combo.clone()).seed(seed), combo);
            let picked: Vec<_> = commands
                .iter()
                .filter(|c| c.starts_with("/time set"))
                .collect();
            assert_eq!(picked.len(), 1, "exactly one time: {commands:?}");
            assert!(picked[0] == night || picked[0] == noon, "{picked:?}");
            let _ = seen.insert(picked[0].clone());
        }
        assert_eq!(seen.len(), 2, "both come up over 20 seeds");
    }

    #[tokio::test]
    async fn a_rotate_command_ends_the_race_and_moves_on() {
        let (mut world, mut director) = two_racing(RaceDirector::new(cycle())).await;
        assert_eq!(director.combo().map(|c| c.track), Some(fe1x()));

        let cx = Context::new(&world, Instant::now());
        let mut effects = director.command(DirectorCommand::Rotate, &cx);
        let commands: Vec<_> = effects
            .packets_sent()
            .filter_map(|p| match p {
                Packet::Mst(Mst { msg, .. }) => Some(msg.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(commands, ["/end"]);
        assert!(director.loading());

        // A second request while it loads does nothing.
        assert!(director.command(DirectorCommand::Rotate, &cx).is_empty());

        // The next combo arrives and is announced.
        let [load] = effects.take_tasks().try_into().ok().expect("one load");
        let loaded = serve(&mut world, &mut director, Input::Msg(load.await)).await;
        assert_eq!(loaded.chat, ["^3Next track: ^7BL1"]);
        assert_eq!(director.combo().map(|c| c.track), Some(bl1()));
    }

    /// The car restrictions sent setting up a lobby already on `combo`'s track:
    /// `/allowmods` and `/unapproved`, the `Plc`s and the `Mal`s.
    fn restrictions(combo: Combo) -> (Vec<String>, Vec<Plc>, Vec<Mal>) {
        let mut world = World::new();
        let lobby = Sta {
            track: combo.track,
            raceinprog: RaceInProgress::No,
            ..Default::default()
        };
        let _ = world.apply(&lobby.into());
        let director = RaceDirector::new(combo.clone());
        let (effects, done) = director.configure::<()>(&combo, None, &world);
        assert!(done, "already on the track");
        let mut seen = (Vec::new(), Vec::new(), Vec::new());
        for packet in effects.packets_sent() {
            match packet {
                Packet::Mst(Mst { msg, .. })
                    if msg.starts_with("/allowmods") || msg.starts_with("/unapproved") =>
                {
                    seen.0.push(msg.clone());
                },
                Packet::Plc(plc) => seen.1.push(plc.clone()),
                Packet::Mal(mal) => seen.2.push(mal.clone()),
                _ => {},
            }
        }
        seen
    }

    #[test]
    fn every_combo_states_its_cars() {
        // By default, everything: whatever the last track restricted is lifted.
        let (allowmods, plcs, mals) = restrictions(Combo::untimed(fe1x()));
        assert_eq!(allowmods, ["/allowmods yes", "/unapproved yes"]);
        let [open] = plcs.try_into().expect("one Plc");
        assert_eq!(open.ucid, ConnectionId::ALL);
        assert_eq!(open.cars, PlcAllowedCarsSet::all());
        let [any] = mals.try_into().expect("one Mal");
        assert!(any.is_empty(), "an empty Mal allows any mod");

        // Standard cars only: those, and no mods at all.
        let rallycross = Combo::untimed(bl1()).vehicles(vec![Vehicle::Uf1, Vehicle::Xfr]);
        let (allowmods, plcs, mals) = restrictions(rallycross);
        // No mod list, so nothing to say about unapproved mods.
        assert_eq!(allowmods, ["/allowmods no"]);
        let [narrow] = plcs.try_into().expect("one Plc");
        assert!(narrow.cars.contains(&Vehicle::Uf1) && narrow.cars.contains(&Vehicle::Xfr));
        assert!(!narrow.cars.contains(&Vehicle::Fxo));
        assert!(mals.is_empty(), "{mals:?}");

        // A mod in the list: mods on, just that one.
        let modded = Combo::untimed(bl1()).vehicles(vec![Vehicle::Uf1, Vehicle::Mod(0x1D_FF72)]);
        let (allowmods, plcs, mals) = restrictions(modded);
        assert_eq!(allowmods, ["/allowmods yes", "/unapproved yes"]);
        let [cars] = plcs.try_into().expect("one Plc");
        assert!(cars.cars.contains(&Vehicle::Uf1) && !cars.cars.contains(&Vehicle::Xfr));
        let [mods] = mals.try_into().expect("one Mal");
        assert!(mods.contains(&Vehicle::Mod(0x1D_FF72)));
        assert_eq!(mods.iter().count(), 1);
    }

    #[test]
    fn a_pit_stop_is_only_required_where_there_is_room_for_one() {
        let asking = |laps| {
            Combo::untimed(fe1x())
                .laps(laps)
                .must_pit(true)
                .pit_stop_required()
        };
        assert!(!asking(RaceLaps::Untimed));
        assert!(!asking(RaceLaps::Practice));
        assert!(!asking(RaceLaps::Laps(1)));
        assert!(asking(RaceLaps::Laps(2)));
        assert!(asking(RaceLaps::Hours(1)));
        // Never when not asked for.
        assert!(
            !Combo::untimed(fe1x())
                .laps(RaceLaps::Laps(5))
                .pit_stop_required()
        );

        // An untimed combo that asks still sends "no".
        let untimed = set_up_commands(Combo::untimed(fe1x()).must_pit(true));
        assert!(untimed.iter().any(|c| c == "/mustpit no"), "{untimed:?}");
    }

    #[tokio::test]
    async fn the_status_names_the_combo_on_the_server_until_the_next_is_set_up() {
        let (mut world, mut director) = two_racing(RaceDirector::new(cycle())).await;
        let on_server = |director: &RaceDirector| director.status().combo.map(|c| c.track);
        assert_eq!(on_server(&director), Some(fe1x()));
        assert_eq!(director.status().next.map(|c| c.track), Some(bl1()));

        // Rotating loads BL1 at once, but FE1X is still on the server.
        let cx = Context::new(&world, Instant::now());
        let mut effects = director.command(DirectorCommand::Rotate, &cx);
        let [load] = effects.take_tasks().try_into().ok().expect("one load");
        let _ = serve(&mut world, &mut director, Input::Msg(load.await)).await;
        assert_eq!(director.combo().map(|c| c.track), Some(bl1()));
        assert_eq!(on_server(&director), Some(fe1x()));

        // The lobby asks for BL1; once it's loaded, BL1 is set up.
        let _ = serve(&mut world, &mut director, lobby(fe1x())).await;
        assert_eq!(on_server(&director), Some(fe1x()));
        let _ = serve(&mut world, &mut director, lobby(bl1())).await;
        assert_eq!(on_server(&director), Some(bl1()));
    }

    #[test]
    fn every_combo_restates_its_race_settings() {
        // Defaults are sent too, so nothing carries over from the last track.
        let plain = set_up_commands(Combo::untimed(fe1x()));
        for want in ["/start finish", "/mustpit no"] {
            assert!(plain.iter().any(|c| c == want), "{want}: {plain:?}");
        }
        let set = set_up_commands(
            Combo::untimed(fe1x())
                .laps(RaceLaps::Laps(5))
                .start(StartOrder::Reverse)
                .must_pit(true),
        );
        for want in ["/start reverse", "/mustpit yes"] {
            assert!(set.iter().any(|c| c == want), "{want}: {set:?}");
        }
    }
}
