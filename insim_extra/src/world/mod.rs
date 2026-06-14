//! [`World`] is an aggregate over [`Presence`], [`Game`], and [`RaceTracker`]
//! that processes them in a single call with no inter-cycle lag.
//!
//! Instead of registering three separate handlers in the correct order, create
//! one `World` and call [`apply_packet`](World::apply_packet) per incoming
//! packet. Presence events are routed directly into the race tracker in the
//! same call, eliminating the two-cycle lag that the split three-handler
//! approach requires.
//!
//! ```ignore
//! let world = World::new();
//!
//! while let Some(packet) = conn.next().await {
//!     for event in world.apply_packet(&packet) {
//!         match event {
//!             WorldEvent::Presence(pe) => { /* ... */ }
//!             WorldEvent::Game(ge) => { /* ... */ }
//!             WorldEvent::Race(re) => { /* ... */ }
//!         }
//!     }
//! }
//! ```

use crate::{
    game::{Game, GameEvent},
    presence::{Presence, PresenceEvent},
    race::{RaceEvent, RaceTracker},
};

/// Aggregate event produced by [`World::apply_packet`].
#[derive(Debug, Clone)]
pub enum WorldEvent {
    /// A presence change (connection joined/left, player joined/left, etc.).
    Presence(PresenceEvent),
    /// A game-state change (session started/ended, track changed, etc.).
    Game(GameEvent),
    /// A race event (entrant joined, lap completed, finished, etc.).
    Race(RaceEvent),
}

/// Aggregate state mirror combining [`Presence`], [`Game`], and [`RaceTracker`].
///
/// State lives behind `Arc<RwLock<…>>` in each component; clones are cheap
/// and share the same underlying maps.
///
/// Use [`World::new()`] for short-form races and qualifying, where every player
/// join is a fresh entrant. Use [`World::with_rejoin()`] for long-form races
/// (endurance / multi-hour) where a player may briefly disconnect and reconnect.
#[derive(Clone, Default)]
pub struct World {
    presence: Presence,
    game: Game,
    race: RaceTracker,
    rejoin: bool,
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("presence", &self.presence)
            .field("game", &self.game)
            .field("race", &self.race)
            .field("rejoin", &self.rejoin)
            .finish()
    }
}

impl World {
    /// Create a new world with empty state. Every player join creates a fresh
    /// entrant - correct for sprints, qualifying, and practice.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new world in rejoin mode. When a player joins, the tracker
    /// first attempts to resume a prior disconnected entrant (matched by LFS.net
    /// username), falling back to a fresh entrant if none is found. Use this for
    /// endurance / multi-hour races where mid-race reconnects should not create
    /// phantom duplicate entrants.
    pub fn with_rejoin() -> Self {
        Self {
            rejoin: true,
            ..Self::default()
        }
    }

    /// Apply one raw packet, update all internal state, and return any events.
    ///
    /// Processing order:
    /// 1. [`Presence`] - connection and player events
    /// 2. Each [`PresenceEvent`] is immediately routed into the race tracker
    ///    (no deferred cycle), producing [`RaceEvent`]s inline.
    /// 3. [`Game`] - session and track events
    /// 4. Each [`GameEvent`] is immediately routed into the race tracker.
    /// 5. Timing packets (`Lap`, `Spx`, `Fin`, `Res`, `Pit`, `Psf`, `Pen`,
    ///    `Plp`, `Reo`) are routed directly to the race tracker.
    pub fn apply_packet(&self, packet: &insim::Packet) -> Vec<WorldEvent> {
        let mut events = Vec::new();

        for pe in self.presence.apply_packet(packet) {
            let race_events = if self.rejoin {
                if let PresenceEvent::PlayerJoined(ref info) = pe {
                    self.race.apply_player_rejoined(info)
                } else {
                    self.race.apply_presence_event(&pe)
                }
            } else {
                self.race.apply_presence_event(&pe)
            };
            for re in race_events {
                events.push(WorldEvent::Race(re));
            }
            events.push(WorldEvent::Presence(pe));
        }

        for ge in self.game.apply_packet(packet) {
            for re in self.race.apply_game_event(&ge) {
                events.push(WorldEvent::Race(re));
            }
            events.push(WorldEvent::Game(ge));
        }

        for re in self.race.apply_packet(packet) {
            events.push(WorldEvent::Race(re));
        }

        events
    }

    /// Access the underlying [`Presence`] tracker.
    pub fn presence(&self) -> &Presence {
        &self.presence
    }

    /// Access the underlying [`Game`] mirror.
    pub fn game(&self) -> &Game {
        &self.game
    }

    /// Access the underlying [`RaceTracker`].
    pub fn race(&self) -> &RaceTracker {
        &self.race
    }

    /// Number of tracked connections.
    pub fn count(&self) -> usize {
        self.presence.count()
    }

    /// Look up a connection by ID.
    pub fn get(
        &self,
        ucid: insim::identifiers::ConnectionId,
    ) -> Option<crate::presence::ConnectionInfo> {
        self.presence.get(ucid)
    }

    /// Look up a player by player ID.
    pub fn player(
        &self,
        plid: insim::identifiers::PlayerId,
    ) -> Option<crate::presence::PlayerInfo> {
        self.presence.player(plid)
    }

    /// Look up the connection that owns a given player.
    pub fn connection_by_player(
        &self,
        plid: insim::identifiers::PlayerId,
    ) -> Option<crate::presence::ConnectionInfo> {
        self.presence.connection_by_player(plid)
    }

    /// Send a spec command for the given connection (returns `None` if not found).
    pub fn spec(
        &self,
        ucid: insim::identifiers::ConnectionId,
    ) -> Option<insim::Packet> {
        self.presence.spec(ucid)
    }

    /// Clear the penalty for the given connection (returns `None` if not found).
    pub fn clear_penalty(
        &self,
        ucid: insim::identifiers::ConnectionId,
    ) -> Option<insim::Packet> {
        self.presence.clear_penalty(ucid)
    }
}
