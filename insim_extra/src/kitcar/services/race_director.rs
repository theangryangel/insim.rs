//! [`RaceDirector`]: runs the races most mini-games share. It waits for
//! players, sets up the lobby, races, and moves through a [`Rotation`] of combos
//! (track, layout, cars).
//!
//! Start it next to your game. Games don't talk to it. They follow the session
//! in [`World`], which reports races starting and ending whoever caused them:
//! this service, an admin's `/end` or `/restart`, or everyone leaving.
//!
//! ```ignore
//! // A new combo every 3 races, in order.
//! let director = RaceDirector::new(Cycle::new([fe1x, bl1])).every(Every::Races(3));
//! let _director = host.start(director).await?;
//! let _game = host.start(MyGame::new()).await?;
//! ```
//!
//! Pass one [`Combo`] to keep it forever. For shuffling, votes or layouts from
//! disk, pass a closure; see [`Rotation`].

use std::{fmt, future::Future, time::Duration};

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
    kitcar::{Context, Effects, Input, Service},
    world::{
        World, WorldEvent,
        race::{FinishStatus, RaceEvent},
    },
};

/// Where the round is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
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
    /// Vehicles allowed for everyone (empty allows all). `None` leaves
    /// restrictions alone, e.g. when another service restricts cars per player.
    pub vehicles: Option<Vec<Vehicle>>,
}

impl Combo {
    /// An untimed session on `track`: no wind, no layout, car restrictions
    /// untouched.
    pub fn untimed(track: Track) -> Self {
        Self {
            track,
            layout: None,
            laps: RaceLaps::Untimed,
            wind: 0,
            vehicles: None,
        }
    }
    /// Load this layout.
    pub fn layout(mut self, layout: Option<Layout>) -> Self {
        self.layout = layout;
        self
    }
    /// Allow only these vehicles, for everyone. Empty allows all.
    pub fn vehicles(mut self, vehicles: Vec<Vehicle>) -> Self {
        self.vehicles = Some(vehicles);
        self
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
/// - [`Cycle`], a fixed list in order;
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
}

/// A fixed list of combos, in order, looping after the last.
#[derive(Debug, Clone)]
pub struct Cycle {
    combos: Vec<Combo>,
    next: usize,
}

impl Cycle {
    /// Cycle through `combos`.
    ///
    /// # Panics
    ///
    /// If `combos` is empty.
    pub fn new(combos: impl IntoIterator<Item = Combo>) -> Self {
        let combos: Vec<Combo> = combos.into_iter().collect();
        assert!(!combos.is_empty(), "a cycle needs at least one combo");
        Self { combos, next: 0 }
    }
}

impl Rotation for Cycle {
    fn next(
        &mut self,
        _: Option<&Combo>,
        _: &World,
    ) -> BoxFuture<'static, Result<Combo, RotationError>> {
        let combo = self.combos[self.next].clone();
        self.next = (self.next + 1) % self.combos.len();
        future::ready(Ok(combo)).boxed()
    }
}

/// The [`RaceDirector`]'s own messages: its timers, and combos as they load.
#[derive(Debug, Clone)]
pub struct RaceDirectorMsg(DirectorMsg);

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
    /// The lobby must wait while a combo loads.
    loading: bool,
    phase: Phase,
    configured: bool,
    ending_session: bool,
    /// Races ended on the current combo, for [`Every::Races`].
    races: u32,
    /// Whether the [`Every::Duration`] timer is running for the current combo.
    timed: bool,
    /// The grace period after the first finisher, if auto-restarting.
    auto_restart: Option<Duration>,
    /// Someone has finished, and the grace period is running.
    finishing: bool,
    /// `/restart` or `/end` is sent; ignore finishes until the session ends.
    moved_on: bool,
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
            loading: false,
            phase: Phase::Waiting,
            configured: false,
            ending_session: false,
            races: 0,
            timed: false,
            auto_restart: None,
            finishing: false,
            moved_on: false,
        }
    }

    /// Move on to the next combo this often.
    pub fn every(mut self, every: Every) -> Self {
        self.every = Some(every);
        self
    }

    /// Restart finished races for the players, who would otherwise wait on the
    /// results screen for an admin. The first finisher starts a `grace`
    /// period; once everyone is home or it runs out, the race restarts, or
    /// returns to the lobby if the next combo is due.
    pub fn auto_restart(mut self, grace: Duration) -> Self {
        self.auto_restart = Some(grace);
        self
    }

    /// Where the round is.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// The current combo, once loaded.
    pub fn combo(&self) -> Option<&Combo> {
        self.current.as_ref()
    }

    /// Whether a race is running.
    pub fn racing(&self) -> bool {
        self.phase == Phase::Racing
    }

    /// Ask the rotation for the next combo. It arrives as a message.
    fn load_next(&mut self, world: &World) -> Effects<RaceDirectorMsg> {
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
        let effects = Effects::new().cancel_all::<RotationTimer>();
        let effects = match result {
            Ok(combo) => {
                let first = self.current.is_none();
                let track = combo.track;
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
    fn follow(&mut self, transition: Transition, world: &World) -> Effects<RaceDirectorMsg> {
        match (transition, self.every) {
            (Transition::Racing, Some(Every::Duration(after))) if !self.timed => {
                self.timed = true;
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

    /// With auto-restart on, start the grace period at the first finisher, and
    /// move on once everyone still on track is home.
    fn watch_finish(
        &mut self,
        input: &Input<RaceDirectorMsg>,
        world: &World,
    ) -> Effects<RaceDirectorMsg> {
        let Some(grace) = self.auto_restart else {
            return Effects::none();
        };
        if self.phase != Phase::Racing {
            self.finishing = false;
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
        let effects = if finished && !self.finishing {
            self.finishing = true;
            Effects::new()
                .schedule_after(Grace, grace, RaceDirectorMsg(DirectorMsg::GraceUp))
                .message(
                    format!("^3Next race in {}s", grace.as_secs()),
                    ConnectionId::ALL,
                )
        } else {
            Effects::none()
        };
        let home = world
            .live_entrants()
            .all(|entrant| matches!(entrant.status, FinishStatus::Finished { .. }));
        if self.finishing && home {
            effects.and(self.move_on())
        } else {
            effects
        }
    }

    /// Restart, or return to the lobby if the next combo is due.
    fn move_on(&mut self) -> Effects<RaceDirectorMsg> {
        self.finishing = false;
        self.moved_on = true;
        // The race being ended counts towards `Every::Races`.
        let due = matches!(self.every, Some(Every::Races(races)) if self.races + 1 >= races);
        let next = if due || self.loading || !self.configured {
            Mst::new("/end")
        } else {
            Mst::new("/restart")
        };
        Effects::new().cancel(Grace).packet(next)
    }

    /// Move on, ending the race if one is running.
    fn time_up(&mut self, world: &World) -> Effects<RaceDirectorMsg> {
        if self.loading {
            return Effects::none();
        }
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
                self.phase = Phase::Racing;
                return (Effects::none(), Some(Transition::Racing));
            },
            Some(WorldEvent::SessionEnded(_)) if self.phase == Phase::Racing => {
                self.phase = Phase::Waiting;
                return (
                    Effects::none(),
                    Some(Transition::Ended { shortfall: false }),
                );
            },
            Some(WorldEvent::Disconnected(_)) if world.connection_count() == 0 => {
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
        if in_lobby && (lobby_ready || (!self.configured && track_changed)) {
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
        let (effects, done) = self.configure(combo, world);
        self.configured = done;
        if done {
            effects.message(READY, ConnectionId::ALL)
        } else {
            effects
        }
    }

    /// Returns the packets and whether setup is complete. LFS changes track
    /// asynchronously, so nothing else is sent until `TrackChanged`.
    fn configure<M: Send + 'static>(&self, combo: &Combo, world: &World) -> (Effects<M>, bool) {
        if world.track() != Some(combo.track) {
            return (
                Effects::new()
                    .packet(Mst::new("/clear"))
                    .packet(Mst::new(format!("/track {}", combo.track))),
                false,
            );
        }
        let effects = Effects::new()
            .packet(Mst::new(combo.laps.to_command()))
            .packet(Mst::new(format!("/wind {}", combo.wind)))
            .packets(
                combo
                    .vehicles
                    .as_deref()
                    .map(restrict_everyone)
                    .unwrap_or_default(),
            )
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
/// cars go in a `Plc` to every connection; mods in a server-wide `Mal`.
fn restrict_everyone(vehicles: &[Vehicle]) -> Vec<insim::Packet> {
    let mut mods = Mal::default();
    let cars = if vehicles.is_empty() {
        PlcAllowedCarsSet::all()
    } else {
        let mut cars = PlcAllowedCarsSet::default();
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
        cars
    };
    vec![
        Plc {
            ucid: ConnectionId::ALL,
            cars,
            ..Default::default()
        }
        .into(),
        mods.into(),
    ]
}

impl Service for RaceDirector {
    type Msg = RaceDirectorMsg;

    fn update(
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
                    Some(transition) => effects.and(self.follow(transition, world)),
                    None => effects,
                };
                effects.and(self.watch_finish(&input, world))
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
        insim::{Cnl, Fin, Mst, Ncn, Npl, Pll, RaceInProgress, Rst, Sta},
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

        let second = serve(&mut world, &mut director, fin(2)).await;
        assert_eq!(second.commands, ["/restart"]);

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
        assert_eq!(left.commands, ["/restart"]);
    }

    #[tokio::test]
    async fn auto_restart_returns_to_the_lobby_when_the_next_combo_is_due() {
        let director = RaceDirector::new(cycle())
            .every(Every::Races(1))
            .auto_restart(Duration::from_secs(60));
        let (mut world, mut director) = two_racing(director).await;

        let _ = serve(&mut world, &mut director, fin(1)).await;
        let home = serve(&mut world, &mut director, fin(2)).await;
        assert_eq!(home.commands, ["/end"]);

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
}
