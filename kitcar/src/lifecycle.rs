//! [`Lifecycle`]: the round most mini-games share. Wait for players, set the track
//! up in the lobby, race, then go back to waiting.
//!
//! A component, not a wrapper: the game holds it as a field, passes it every
//! input and reacts to the [`Transition`]s it reports, so each game keeps control
//! of its own flow. A game with a different shape (rounds, no race at all) simply
//! doesn't use it.
//!
//! ```ignore
//! fn update(&mut self, input: Input<Msg>, cx: &Context) -> Effects<Msg> {
//!     let (setup, transition) = self.lifecycle.update(&input, cx);
//!     let fx = match transition {
//!         Some(Transition::Racing) => Effects::new().message("Go!", ConnectionId::ALL),
//!         Some(Transition::Ended { .. }) => self.score_everyone(cx),
//!         _ => Effects::none(),
//!     };
//!     setup.and(fx).and(/* the game's own handling, using self.lifecycle.racing() */)
//! }
//! ```

use std::fmt;

use insim::{
    core::{object::ObjectInfo, track::Track, vehicle::Vehicle},
    identifiers::ConnectionId,
    insim::{Axm, PmoAction, PmoFlags, RaceLaps},
};
use insim_extra::world::{World, WorldEvent};

use crate::{Context, Effects, Input};

/// Where the round is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    /// Waiting for players, or for the lobby.
    #[default]
    Waiting,
    /// Setting the track up in the lobby; players type `/ready` to start.
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

/// A change of phase the game should react to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    /// The lobby is being set up.
    SetupStarted,
    /// The race started.
    Racing,
    /// The race ended; `shortfall` if everyone left rather than the session ending.
    Ended {
        /// Everyone left, rather than the session ending normally.
        shortfall: bool,
    },
}

/// The layout a [`Setup`] loads.
#[derive(Debug, Clone)]
pub enum Layout {
    /// A layout saved on the server, loaded with `/axload`.
    Named(String),
    /// Objects sent by InSim, such as from a `.lyt` file.
    Objects(Vec<ObjectInfo>),
}

/// How the server is set up before each race.
#[derive(Debug, Clone)]
pub struct Setup {
    /// The track to change to, if it is not already loaded.
    pub track: Track,
    /// The layout to load, or `None` to clear it.
    pub layout: Option<Layout>,
    /// Race length.
    pub laps: RaceLaps,
    /// Wind strength (0 for none).
    pub wind: u8,
    /// Allowed vehicles; empty allows everything.
    pub vehicles: Vec<Vehicle>,
}

impl Setup {
    /// An untimed session on `track`, with no wind, no layout and any vehicle.
    pub fn untimed(track: Track) -> Self {
        Self {
            track,
            layout: None,
            laps: RaceLaps::Untimed,
            wind: 0,
            vehicles: Vec::new(),
        }
    }
    /// Load this layout.
    pub fn layout(mut self, layout: Option<Layout>) -> Self {
        self.layout = layout;
        self
    }
    /// Allow only these vehicles.
    pub fn vehicles(mut self, vehicles: Vec<Vehicle>) -> Self {
        self.vehicles = vehicles;
        self
    }
}

/// Runs the shared round: see the [module docs](self).
///
/// Multiplayer only, like the rest of kitcar: it counts players with
/// [`World::connection_count`], which leaves out the server's own connection.
#[derive(Debug)]
pub struct Lifecycle {
    setup: Setup,
    phase: Phase,
    configured: bool,
    ending_session: bool,
}

const READY: &str = "^2Setup complete - type ^3/ready^2 to start!";

impl Lifecycle {
    /// Set the server up with `setup` before each race.
    pub fn new(setup: Setup) -> Self {
        Self {
            setup,
            phase: Phase::Waiting,
            configured: false,
            ending_session: false,
        }
    }

    /// Where the round is.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// The setup applied before each race.
    pub fn setup(&self) -> &Setup {
        &self.setup
    }

    /// Use `setup` from the next lobby on, changing track first if needed. This
    /// is how a track rotation plugs in: pick the next setup on
    /// [`Transition::Ended`].
    pub fn set_setup(&mut self, setup: Setup) {
        self.setup = setup;
        self.configured = false;
    }

    /// Whether a race is running.
    pub fn racing(&self) -> bool {
        self.phase == Phase::Racing
    }

    /// Pass every input. Returns the packets to send, and the transition, if any.
    pub fn update<M: Send + 'static>(
        &mut self,
        input: &Input<M>,
        cx: &Context<'_>,
    ) -> (Effects<M>, Option<Transition>) {
        let world = cx.world();
        match input.world() {
            Some(WorldEvent::SessionStarted(_)) if self.phase == Phase::SettingUp => {
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
                        Effects::new().packet(world.end()),
                        Some(Transition::Ended { shortfall: true }),
                    ),
                    _ => (Effects::none(), None),
                };
            },
            _ => {},
        }

        // Until the startup Sta arrives, an unknown session is not proof of the
        // lobby, so reconcile from the settled world rather than the event alone.
        let lobby_ready = matches!(input.world(), Some(WorldEvent::LobbyReady(_)));
        let track_changed = matches!(input.world(), Some(WorldEvent::TrackChanged(_)));
        let in_lobby =
            world.track().is_some() && world.session().is_none() && world.connection_count() > 0;
        if in_lobby && (lobby_ready || (!self.configured && track_changed)) {
            self.ending_session = false;
            self.phase = Phase::SettingUp;
            if self.configured {
                return (
                    Effects::new().message(READY, ConnectionId::ALL),
                    Some(Transition::SetupStarted),
                );
            }
            let (effects, done) = self.configure(world);
            self.configured = done;
            let effects = if done {
                effects.message(READY, ConnectionId::ALL)
            } else {
                effects
            };
            return (effects, Some(Transition::SetupStarted));
        }

        // Taking over a server mid-session: end it once, and let the lobby that
        // follows drive the setup.
        if !self.configured
            && !self.ending_session
            && world.session().is_some()
            && world.connection_count() > 0
        {
            self.ending_session = true;
            self.phase = Phase::SettingUp;
            return (
                Effects::new().packet(world.end()),
                Some(Transition::SetupStarted),
            );
        }
        (Effects::none(), None)
    }

    /// Returns the packets, and whether setup is complete. Changing track is
    /// asynchronous in LFS, so nothing else is sent until `TrackChanged`.
    fn configure<M: Send + 'static>(&self, world: &World) -> (Effects<M>, bool) {
        if world.track() != Some(self.setup.track) {
            return (
                Effects::new()
                    .packet(world.clear())
                    .packet(world.change_track(self.setup.track)),
                false,
            );
        }
        let effects = Effects::new()
            .packet(world.change_laps(self.setup.laps))
            .packet(world.change_wind(self.setup.wind))
            .packets(world.restrict_vehicles(&self.setup.vehicles))
            .packet(world.ax_clear());
        let effects = match &self.setup.layout {
            None => effects,
            Some(Layout::Named(name)) => effects
                .message(
                    "^3Setting up - loading layout, please wait...",
                    ConnectionId::ALL,
                )
                .packet(world.ax_load(name.clone())),
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

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use insim::{
        Packet,
        insim::{Cnl, Mst, Ncn, RaceInProgress, Rst, Sta},
    };

    use super::*;

    /// Apply a packet to the world and pass the packet and its events on, as the
    /// host does. Returns the commands sent and the transitions reported.
    fn feed(
        world: &mut World,
        lifecycle: &mut Lifecycle,
        packet: Packet,
    ) -> (Vec<String>, Vec<Transition>) {
        let events = world.apply_packet(&packet);
        let world = &*world;
        let cx = Context::new(world, Instant::now());
        let mut commands = Vec::new();
        let mut transitions = Vec::new();
        let inputs = std::iter::once(Input::<()>::Packet(packet))
            .chain(events.into_iter().map(Input::World));
        for input in inputs {
            let (effects, transition) = lifecycle.update(&input, &cx);
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
        let mut lifecycle = Lifecycle::new(Setup::untimed(track));

        // The lobby, already on the right track.
        let _ = feed(
            &mut world,
            &mut lifecycle,
            Sta {
                track,
                raceinprog: RaceInProgress::No,
                ..Default::default()
            }
            .into(),
        );
        let (commands, transitions) = feed(
            &mut world,
            &mut lifecycle,
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
        assert_eq!(lifecycle.phase(), Phase::SettingUp);

        // /ready starts the session.
        let (_, transitions) = feed(
            &mut world,
            &mut lifecycle,
            Rst {
                track,
                ..Default::default()
            }
            .into(),
        );
        assert_eq!(transitions, [Transition::Racing]);
        assert!(lifecycle.racing());

        // Everyone leaves mid-race: end it.
        let (commands, transitions) = feed(
            &mut world,
            &mut lifecycle,
            Cnl {
                ucid: ConnectionId(1),
                ..Default::default()
            }
            .into(),
        );
        assert_eq!(transitions, [Transition::Ended { shortfall: true }]);
        assert_eq!(commands, ["/end"]);
        assert_eq!(lifecycle.phase(), Phase::Waiting);
        // A rotation picks the next track; once LFS is back in the lobby (after
        // the /end), the next player to join triggers the change.
        lifecycle.set_setup(Setup::untimed("BL1".parse().unwrap()));
        let _ = feed(
            &mut world,
            &mut lifecycle,
            Sta {
                track,
                raceinprog: RaceInProgress::No,
                ..Default::default()
            }
            .into(),
        );
        let (commands, _) = feed(
            &mut world,
            &mut lifecycle,
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
        let mut lifecycle = Lifecycle::new(Setup::untimed("BL1".parse().unwrap()));
        let _ = feed(
            &mut world,
            &mut lifecycle,
            Sta {
                track: "FE1X".parse().unwrap(),
                ..Default::default()
            }
            .into(),
        );
        let (commands, transitions) = feed(
            &mut world,
            &mut lifecycle,
            Ncn {
                ucid: ConnectionId(1),
                ..Default::default()
            }
            .into(),
        );
        assert_eq!(transitions, [Transition::SetupStarted]);
        assert_eq!(commands, ["/clear", "/track BL1"]);
    }
}
