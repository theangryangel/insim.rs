//! [`Game`] mirrors game state from a bare `insim` packet stream.
//!
//! Host commands return [`insim::Packet`] values; multi-packet commands return
//! [`Vec<insim::Packet>`]. Feed packets with [`apply`](Game::apply) (state only) or
//! [`apply_events`](Game::apply_events) (state + change events).
//!
//! ```ignore
//! let game = Game::new();
//!
//! while let Some(packet) = conn.next().await {
//!     for event in game.apply_events(&packet) {
//!         match event {
//!             GameEvent::RaceStarted => println!("race started!"),
//!             GameEvent::TrackChanged { to, .. } => println!("track: {to}"),
//!             _ => {}
//!         }
//!     }
//! }
//! ```

use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use insim::{
    core::{track::Track, vehicle::Vehicle, wind::Wind},
    identifiers::ConnectionId,
    insim::{
        Axi, Ism, Mal, Plc, PlcAllowedCarsSet, RaceInProgress, RaceLaps, Sta, StaFlags, Tiny,
        TinyType,
    },
};
use tokio_util::sync::CancellationToken;

use crate::util::host_command;

/// Whether LFS is currently running in local or multiplayer mode.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum MultiplayerState {
    /// LFS is in single-player / offline mode.
    #[default]
    Local,
    /// LFS is connected to a multiplayer session.
    Multiplayer {
        /// Host name of the server.
        host_name: String,
        /// `true` if this instance is the host; `false` if a guest.
        is_host: bool,
    },
}

/// Mirror of the relevant fields from an `Sta` packet.
#[derive(Debug, Default, Clone)]
pub struct GameInfo {
    track: Option<Track>,
    layout: Option<String>,
    weather: Option<u8>,
    wind: Option<Wind>,
    racing: RaceInProgress,
    qualifying_duration: Duration,
    race_duration: RaceLaps,
    flags: StaFlags,
    multiplayer: MultiplayerState,
}

impl GameInfo {
    /// Currently selected track, if known.
    pub fn current_track(&self) -> Option<&Track> {
        self.track.as_ref()
    }

    /// Currently loaded layout, if known.
    pub fn current_layout(&self) -> Option<&String> {
        self.layout.as_ref()
    }

    /// Weather identifier (0..=2 typically).
    pub fn weather(&self) -> Option<u8> {
        self.weather
    }

    /// Wind conditions.
    pub fn wind(&self) -> Option<&Wind> {
        self.wind.as_ref()
    }

    /// Race-in-progress state.
    pub fn racing(&self) -> &RaceInProgress {
        &self.racing
    }

    /// Qualifying duration.
    pub fn qualifying_duration(&self) -> Duration {
        self.qualifying_duration
    }

    /// Race-laps configuration.
    pub fn race_duration(&self) -> &RaceLaps {
        &self.race_duration
    }

    /// Overall game flags.
    pub fn flags(&self) -> &StaFlags {
        &self.flags
    }

    /// Current multiplayer state.
    ///
    /// [`MultiplayerState::Local`] until an `ISM` packet is received or when
    /// LFS is not in multiplayer mode (empty host name in the `ISM`).
    pub fn multiplayer(&self) -> &MultiplayerState {
        &self.multiplayer
    }
}

/// State-change events produced by [`Game::apply_events`].
///
/// Standalone users pattern-match this directly. `kitcar` users receive
/// individual typed events via `Event<T>` extractors.
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// Race transitioned from non-racing → racing.
    RaceStarted,
    /// Race transitioned from racing → non-racing.
    RaceEnded,
    /// Track changed (also fired for the first `Sta` when `from` is `None`).
    TrackChanged {
        /// Previously known track.
        from: Option<Track>,
        /// New track.
        to: Track,
    },
    /// Layout changed or cleared.
    LayoutChanged {
        /// Previously known layout.
        from: Option<String>,
        /// New layout, or `None` if cleared.
        to: Option<String>,
    },
    /// LFS joined or started a multiplayer session.
    ///
    /// Fired on the first `ISM` with a non-empty host name, and again if the
    /// host name changes (e.g. reconnecting to a different server).
    MultiplayerJoined {
        /// Multiplayer host name.
        host_name: String,
        /// `true` if this instance is the host.
        is_host: bool,
    },
    /// LFS left multiplayer (received an `ISM` with an empty host name).
    MultiplayerLeft,
}

/// Mirrors game state from a stream of `insim` packets.
#[derive(Clone)]
pub struct Game {
    inner: Arc<RwLock<GameInfo>>,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Game").finish_non_exhaustive()
    }
}

impl Game {
    /// Create a new game mirror with empty state.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(GameInfo::default())),
        }
    }

    /// Snapshot of the current game state.
    pub fn get(&self) -> GameInfo {
        self.inner.read().expect("poison").clone()
    }

    /// `/end` - finish the current race.
    pub fn end(&self) -> insim::Packet {
        host_command("/end")
    }

    /// `/track {track}` - load a different track.
    pub fn change_track(&self, track: Track) -> insim::Packet {
        host_command(format!("/track {track}"))
    }

    /// Change race length. Maps onto `/laps`, `/hours`, or `/laps no`.
    pub fn change_laps(&self, laps: RaceLaps) -> insim::Packet {
        let cmd = match laps {
            RaceLaps::Untimed => "/laps no".to_string(),
            RaceLaps::Hours(h) => format!("/hours {h}"),
            other => format!("/laps {}", Into::<u8>::into(other)),
        };
        host_command(cmd)
    }

    /// `/wind {wind}` - set wind strength (0..=2 typically).
    pub fn change_wind(&self, wind: u8) -> insim::Packet {
        host_command(format!("/wind {wind}"))
    }

    /// `/axclear` - clear the autocross layout.
    pub fn ax_clear(&self) -> insim::Packet {
        host_command("/axclear")
    }

    /// `/axload {layout}` - load an autocross layout by name.
    pub fn ax_load(&self, layout: impl Into<String>) -> insim::Packet {
        host_command(format!("/axload {}", layout.into()))
    }

    /// `/restart` - start a race.
    pub fn restart(&self) -> insim::Packet {
        host_command("/restart")
    }

    /// `/qualify` - start qualifying.
    pub fn qualify(&self) -> insim::Packet {
        host_command("/qualify")
    }

    /// `/reinit` - full restart, kicks all connections.
    pub fn reinit(&self) -> insim::Packet {
        host_command("/reinit")
    }

    /// `/weather {weather}` - set weather/lighting.
    pub fn change_weather(&self, weather: u8) -> insim::Packet {
        host_command(format!("/weather {weather}"))
    }

    /// `/qual {minutes}` - set qualifying duration. `0` = no qualifying.
    pub fn change_qual(&self, minutes: u8) -> insim::Packet {
        host_command(format!("/qual {minutes}"))
    }

    /// `/pit_all` - send every player to the pits.
    pub fn pit_all(&self) -> insim::Packet {
        host_command("/pit_all")
    }

    /// Apply vehicle restrictions server-wide (ucid = `ConnectionId::ALL`).
    ///
    /// Sends a `Plc` packet for standard cars and a `Mal` packet for mods.
    /// Pass an empty slice to clear all restrictions.
    pub fn restrict_vehicles(&self, vehicles: &[Vehicle]) -> Vec<insim::Packet> {
        let mut mal = Mal::default();
        let cars = if vehicles.is_empty() {
            PlcAllowedCarsSet::all()
        } else {
            let mut cars = PlcAllowedCarsSet::default();
            for v in vehicles {
                match v {
                    Vehicle::Mod(_) => {
                        let _ = mal.insert(*v);
                    },
                    _ => {
                        let _ = cars.insert(*v);
                    },
                }
            }
            cars
        };
        vec![
            insim::Packet::from(Plc {
                cars,
                ucid: ConnectionId::ALL,
                ..Plc::default()
            }),
            insim::Packet::from(mal),
        ]
    }

    /// Poll `predicate` against the current state every `poll_interval`
    /// until it returns true. Returns `None` if `cancel` fires first.
    pub async fn wait_for<F: Fn(&GameInfo) -> bool>(
        &self,
        predicate: F,
        poll_interval: Duration,
        cancel: CancellationToken,
    ) -> Option<()> {
        let mut interval = tokio::time::interval(poll_interval);
        loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => return None,
                _ = interval.tick() => {
                    if predicate(&self.get()) {
                        return Some(());
                    }
                }
            }
        }
    }

    /// Wait until the game is no longer in progress.
    pub async fn wait_for_end(&self, cancel: CancellationToken) -> Option<()> {
        self.wait_for(
            |info| !info.flags.is_in_game() && matches!(info.racing, RaceInProgress::No),
            Duration::from_millis(500),
            cancel,
        )
        .await
    }

    /// Wait until the given track is loaded and the server is on the
    /// selection screen (not yet racing).
    pub async fn wait_for_track(&self, track: Track, cancel: CancellationToken) -> Option<()> {
        self.wait_for(
            |info| {
                info.track.as_ref() == Some(&track)
                    && !info.flags.is_in_game()
                    && matches!(info.racing, RaceInProgress::No)
            },
            Duration::from_millis(500),
            cancel,
        )
        .await
    }

    /// Wait until racing is actually in progress (all players ready).
    pub async fn wait_for_racing(&self, cancel: CancellationToken) -> Option<()> {
        self.wait_for(
            |info| info.flags.is_in_game() && matches!(info.racing, RaceInProgress::Racing),
            Duration::from_millis(500),
            cancel,
        )
        .await
    }

    /// Wait for a specific layout to be loaded.
    pub async fn wait_for_layout(&self, layout: String, cancel: CancellationToken) -> Option<()> {
        self.wait_for(
            |info| info.layout.as_deref() == Some(layout.as_str()),
            Duration::from_millis(500),
            cancel,
        )
        .await
    }

    /// Apply one raw packet to the internal state mirror without returning
    /// any events.
    pub fn apply(&self, packet: &insim::Packet) {
        let _ = self.apply_events(packet);
    }

    /// Apply one raw packet and return the resulting state-change events.
    pub fn apply_events(&self, packet: &insim::Packet) -> Vec<GameEvent> {
        match packet {
            insim::Packet::Sta(sta) => {
                let (was_racing, now_racing, prev_track, new_track) = self.apply_sta(sta);
                let mut events = Vec::new();
                if !was_racing && now_racing {
                    events.push(GameEvent::RaceStarted);
                }
                if was_racing && !now_racing {
                    events.push(GameEvent::RaceEnded);
                }
                if prev_track != Some(new_track) {
                    events.push(GameEvent::TrackChanged {
                        from: prev_track,
                        to: new_track,
                    });
                }
                events
            },
            insim::Packet::Axi(axi) => {
                let (prev, new) = self.apply_axi(axi);
                if prev != new {
                    vec![GameEvent::LayoutChanged {
                        from: prev,
                        to: new,
                    }]
                } else {
                    vec![]
                }
            },
            insim::Packet::Ism(ism) => {
                let (prev, new) = self.apply_ism(ism);
                if prev == new {
                    return vec![];
                }
                match new {
                    MultiplayerState::Multiplayer { host_name, is_host } => {
                        vec![GameEvent::MultiplayerJoined { host_name, is_host }]
                    },
                    MultiplayerState::Local => vec![GameEvent::MultiplayerLeft],
                }
            },
            insim::Packet::Tiny(tiny) => {
                if let Some(prev) = self.apply_tiny_axc(tiny) {
                    if prev.is_some() {
                        vec![GameEvent::LayoutChanged {
                            from: prev,
                            to: None,
                        }]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            },
            _ => vec![],
        }
    }

    fn apply_sta(&self, sta: &Sta) -> (bool, bool, Option<Track>, Track) {
        let mut g = self.inner.write().expect("poison");
        let was_racing = matches!(g.racing, RaceInProgress::Racing);
        let prev_track = g.track;
        g.racing = sta.raceinprog.clone();
        g.qualifying_duration = Duration::from_secs(sta.qualmins as u64 * 60);
        g.race_duration = sta.racelaps;
        g.track = Some(sta.track);
        g.weather = Some(sta.weather);
        g.wind = Some(sta.wind);
        g.flags = sta.flags;
        let now_racing = matches!(g.racing, RaceInProgress::Racing);
        (was_racing, now_racing, prev_track, sta.track)
    }

    fn apply_axi(&self, axi: &Axi) -> (Option<String>, Option<String>) {
        let mut g = self.inner.write().expect("poison");
        let prev = g.layout.clone();
        g.layout = Some(axi.lname.clone());
        (prev, Some(axi.lname.clone()))
    }

    fn apply_ism(&self, ism: &Ism) -> (MultiplayerState, MultiplayerState) {
        let mut g = self.inner.write().expect("poison");
        let prev = g.multiplayer.clone();
        // NOTE: If LFS is not in multiplayer mode, the host name in the ISM will be empty.
        g.multiplayer = if ism.hname.is_empty() {
            MultiplayerState::Local
        } else {
            MultiplayerState::Multiplayer {
                host_name: ism.hname.clone(),
                is_host: ism.host,
            }
        };
        (prev, g.multiplayer.clone())
    }

    fn apply_tiny_axc(&self, tiny: &Tiny) -> Option<Option<String>> {
        if !matches!(tiny.subt, TinyType::Axc) {
            return None;
        }
        let mut g = self.inner.write().expect("poison");
        let prev = g.layout.clone();
        g.layout = None;
        Some(prev)
    }
}
