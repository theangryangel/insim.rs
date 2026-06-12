//! [`Game`] mirrors game state from a bare `insim` packet stream.
//!
//! Host commands return [`insim::Packet`] values; multi-packet commands return
//! [`Vec<insim::Packet>`]. Feed packets with [`apply_packet`](Game::apply_packet)
//! to update state and collect change events.
//!
//! ```ignore
//! let game = Game::new();
//!
//! while let Some(packet) = conn.next().await {
//!     for event in game.apply_packet(&packet) {
//!         match event {
//!             GameEvent::SessionStarted { kind } => println!("session started: {kind:?}"),
//!             GameEvent::TrackChanged { to, .. } => println!("track: {to}"),
//!             _ => {}
//!         }
//!     }
//! }
//! ```

mod commands;
use std::{sync::Arc, time::Duration};

pub use commands::{GridMode, Month, TimeDemoPreset, TimeSet};
use insim::{
    core::{game_version::GameVersion, track::Track, vehicle::Vehicle, wind::Wind},
    insim::{
        Axi, Ism, PlcAllowedCarsSet, RaceFlags, RaceInProgress, RaceLaps, Rst, SmallType, Sta,
        StaFlags, Tiny, TinyType, Ver,
    },
};
use parking_lot::RwLock;
use tokio_util::sync::CancellationToken;

/// The kind of session that an [`Rst`] packet started.
///
/// Carried by [`GameEvent::SessionStarted`] so consumers (notably the race
/// tracker) can interpret subsequent packets correctly - most importantly
/// `Fin`, which marks a finish in a race but fires after every lap in
/// qualifying.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    /// A race session (`Rst` with `qualmins == 0` and a lap/hour count).
    Race,
    /// A qualifying session (`Rst` with `qualmins > 0`).
    Qualifying,
    /// A practice session (`Rst` with `qualmins == 0` and
    /// [`RaceLaps::Practice`], i.e. 0 laps).
    ///
    /// LFS reports no race in progress, so this is only distinguishable from
    /// the `Rst` packet. Like qualifying, `Fin` here is a per-lap signal rather
    /// than a finish.
    Practice,
    /// An untimed open/cruise session (`Rst` with `qualmins == 0` and
    /// [`RaceLaps::Untimed`]).
    ///
    /// Like [`Practice`](Self::Practice) LFS reports no race in progress, and
    /// `Fin` is a per-lap signal rather than a finish.
    Untimed,
}

/// High-level description of the current LFS session.
#[derive(Debug, Default, Clone)]
pub enum SessionState {
    /// No `Sta` or `Rst` received yet; state unknown.
    #[default]
    Unknown,
    /// No session active - players are on the track-selection or end screen.
    Lobby,
    /// A race session is in progress.
    Racing {
        /// Race length configuration.
        laps: RaceLaps,
        /// Race flags from the `Rst` packet, or empty if only `Sta` has been received.
        flags: RaceFlags,
    },
    /// A qualifying session is in progress.
    Qualifying {
        /// Qualifying duration.
        duration: Duration,
        /// Race flags from the `Rst` packet, or empty if only `Sta` has been received.
        flags: RaceFlags,
    },
}

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

/// Version information about the connected LFS instance, from a `Ver` packet.
#[derive(Debug, Default, Clone)]
pub struct VersionInfo {
    /// Product name (e.g. `"S3"`).
    pub product: String,
    /// LFS game version.
    pub version: GameVersion,
}

/// Mirror of the relevant fields from an `Sta` packet.
#[derive(Debug, Default, Clone)]
pub struct GameInfo {
    track: Option<Track>,
    layout: Option<String>,
    weather: Option<u8>,
    wind: Option<Wind>,
    session: SessionState,
    flags: StaFlags,
    multiplayer: MultiplayerState,
    /// Server's allowed-cars set, from a `Small`/`Alc` reply. `None` until seen.
    allowed_cars: Option<PlcAllowedCarsSet>,
    /// Server's allowed-mods list, from a `Mal` packet. Empty = unrestricted or
    /// not yet received.
    allowed_mods: Vec<Vehicle>,
    /// Version information, from a `Ver` packet. `None` until received.
    version: Option<VersionInfo>,
    /// Incremented each time an `Axi` packet is applied, regardless of lname content.
    axi_count: u64,
    /// Incremented each time an `Rst` packet is applied.
    rst_count: u64,
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

    /// Current session state.
    pub fn session(&self) -> &SessionState {
        &self.session
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

    /// Server's allowed-cars set, if a `Small`/`Alc` has been received.
    ///
    /// Populated by requesting `TinyType::Alc`, or when the host changes the
    /// allowed cars mid-session.
    pub fn allowed_cars(&self) -> Option<&PlcAllowedCarsSet> {
        self.allowed_cars.as_ref()
    }

    /// Server's allowed-mods list, if a `Mal` has been received. Empty means
    /// unrestricted (any mod) or not yet received.
    ///
    /// Populated by requesting `TinyType::Mal`, or when the host changes the
    /// allowed mods mid-session.
    pub fn allowed_mods(&self) -> &[Vehicle] {
        &self.allowed_mods
    }

    /// Version information about the connected LFS instance, if a `Ver` has
    /// been received. LFS sends this on connect when `Isi::reqi` is non-zero,
    /// or in reply to `TinyType::Ver`.
    pub fn version(&self) -> Option<&VersionInfo> {
        self.version.as_ref()
    }
}

/// State-change events produced by [`Game::apply_packet`].
///
/// Standalone users pattern-match this directly. `kitcar` users receive
/// individual typed events via `Event<T>` extractors.
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// An `Rst` packet started a new race, qualifying, practice or untimed
    /// session.
    ///
    /// This is the authoritative session-start signal: LFS sends an
    /// unsolicited `Rst` precisely when a session starts or restarts, carrying
    /// enough information to tell races, qualifying and practice/untimed apart
    /// (see [`SessionKind`]). Consumers that accumulate per-session state
    /// should clear it here.
    SessionStarted {
        /// The kind of session that started.
        kind: SessionKind,
    },
    /// LFS returned to the entry/lobby screen: an `Sta` reported no race or
    /// qualifying in progress where the previous state had one.
    ///
    /// This is the session-end signal - there is no `Rst`-based equivalent, as
    /// `Rst` only ever starts a session. Note that practice/untimed sessions
    /// are reported by LFS as "no race in progress", so this does not fire for
    /// them.
    SessionEnded,
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
    /// The server's allowed-cars set changed (from a `Small`/`Alc`).
    ///
    /// Also fired the first time the set is received.
    AllowedCarsChanged {
        /// The new allowed-cars set.
        cars: PlcAllowedCarsSet,
    },
    /// The server's allowed-mods list changed (from a `Mal`).
    ///
    /// Also fired the first time the list is received.
    AllowedModsChanged {
        /// The new allowed-mods list (empty means unrestricted).
        mods: Vec<Vehicle>,
    },
    /// Version information was received (from a `Ver`).
    VersionReceived {
        /// Product name (e.g. `"S3"`).
        product: String,
        /// LFS game version.
        version: GameVersion,
    },
}

/// Mirrors game state from a stream of `insim` packets.
#[derive(Clone)]
pub struct Game {
    pub(crate) inner: Arc<RwLock<GameInfo>>,
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
    /// Tiny requests to send once on connect to sync game/session state,
    /// allowed cars, and allowed mods. LFS does not send these automatically.
    pub const STARTUP_REQUESTS: &[TinyType] = &[
        TinyType::Sst,
        TinyType::Axi,
        TinyType::Ism,
        TinyType::Alc,
        TinyType::Mal,
    ];

    /// Tiny requests to re-send on each [`GameEvent::SessionStarted`], since a
    /// new session can change the allowed cars/mods.
    pub const SESSION_REQUESTS: &[TinyType] = &[TinyType::Alc, TinyType::Mal];

    /// Create a new game mirror with empty state.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(GameInfo::default())),
        }
    }

    /// Snapshot of the current game state.
    pub fn get(&self) -> GameInfo {
        self.inner.read().clone()
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

    /// Wait until state is populated from at least one `Sta` packet - i.e.
    /// session is no longer [`SessionState::Unknown`] and the current track
    /// is known.
    pub async fn wait_for_known_state(&self, cancel: CancellationToken) -> Option<()> {
        self.wait_for(
            |info| !matches!(info.session, SessionState::Unknown) && info.track.is_some(),
            Duration::from_millis(100),
            cancel,
        )
        .await
    }

    /// Wait until the game is no longer in progress.
    pub async fn wait_for_end(&self, cancel: CancellationToken) -> Option<()> {
        self.wait_for(
            |info| matches!(info.session, SessionState::Lobby),
            Duration::from_millis(500),
            cancel,
        )
        .await
    }

    /// Wait until the given track is loaded and the session is
    /// [`SessionState::Lobby`] (selection screen, not yet racing).
    pub async fn wait_for_track(&self, track: Track, cancel: CancellationToken) -> Option<()> {
        self.wait_for(
            |info| {
                info.track.as_ref() == Some(&track) && matches!(info.session, SessionState::Lobby)
            },
            Duration::from_millis(500),
            cancel,
        )
        .await
    }

    /// Wait until a race session is in progress.
    pub async fn wait_for_racing(&self, cancel: CancellationToken) -> Option<()> {
        self.wait_for(
            |info| matches!(info.session, SessionState::Racing { .. }),
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

    /// Wait for any `Axi` packet to be received.
    ///
    /// Useful when `/axload` is sent but `lname` in the resulting `Axi`
    /// reply is blank (a known LFS behaviour), making it impossible to match
    /// on the layout name.
    pub async fn wait_for_any_axi(&self, cancel: CancellationToken) -> Option<()> {
        let before = self.inner.read().axi_count;
        self.wait_for(
            move |info| info.axi_count != before,
            Duration::from_millis(100),
            cancel,
        )
        .await
    }

    /// Wait for any `Rst` packet to be received, indicating a race or
    /// qualifying session has started.
    pub async fn wait_for_any_rst(&self, cancel: CancellationToken) -> Option<()> {
        let before = self.inner.read().rst_count;
        self.wait_for(
            move |info| info.rst_count != before,
            Duration::from_millis(100),
            cancel,
        )
        .await
    }

    /// Apply one raw packet, update internal state, and return any state-change events.
    pub fn apply_packet(&self, packet: &insim::Packet) -> Vec<GameEvent> {
        match packet {
            insim::Packet::Sta(sta) => {
                let (was_in_session, now_in_session, prev_track, new_track) = self.apply_sta(sta);
                let mut events = Vec::new();
                if was_in_session && !now_in_session {
                    events.push(GameEvent::SessionEnded);
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
                let (prev_lname, new_lname) = self.apply_axi(axi);
                if prev_lname != new_lname {
                    vec![GameEvent::LayoutChanged {
                        from: prev_lname,
                        to: new_lname,
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
            insim::Packet::Tiny(tiny) => self
                .apply_tiny_axc(tiny)
                .map(|prev| GameEvent::LayoutChanged {
                    from: Some(prev),
                    to: None,
                })
                .into_iter()
                .collect(),
            insim::Packet::Rst(rst) => {
                if let Some(kind) = self.apply_rst(rst) {
                    vec![GameEvent::SessionStarted { kind }]
                } else {
                    vec![]
                }
            },
            insim::Packet::Small(small) => {
                if let SmallType::Alc(cars) = &small.subt {
                    if self.apply_allowed_cars(cars) {
                        vec![GameEvent::AllowedCarsChanged { cars: cars.clone() }]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            },
            insim::Packet::Mal(mal) => {
                let mods: Vec<Vehicle> = mal.iter().copied().collect();
                if self.apply_allowed_mods(&mods) {
                    vec![GameEvent::AllowedModsChanged { mods }]
                } else {
                    vec![]
                }
            },
            insim::Packet::Ver(ver) => {
                self.apply_version(ver);
                vec![GameEvent::VersionReceived {
                    product: ver.product.clone(),
                    version: ver.version.clone(),
                }]
            },
            _ => vec![],
        }
    }

    /// Returns `(was_in_session, now_in_session, prev_track, new_track)`, where
    /// "in session" means a race or qualifying session is in progress.
    fn apply_sta(&self, sta: &Sta) -> (bool, bool, Option<Track>, Track) {
        let mut g = self.inner.write();
        let was_in_session = matches!(
            g.session,
            SessionState::Racing { .. } | SessionState::Qualifying { .. }
        );
        let prev_track = g.track;
        g.session = match sta.raceinprog {
            RaceInProgress::No => SessionState::Lobby,
            RaceInProgress::Racing => SessionState::Racing {
                laps: sta.racelaps,
                flags: RaceFlags::empty(),
            },
            RaceInProgress::Qualifying => SessionState::Qualifying {
                duration: Duration::from_secs(sta.qualmins as u64 * 60),
                flags: RaceFlags::empty(),
            },
            _ => SessionState::Unknown,
        };
        g.track = Some(sta.track);
        g.weather = Some(sta.weather);
        g.wind = Some(sta.wind);
        g.flags = sta.flags;
        let now_in_session = matches!(
            g.session,
            SessionState::Racing { .. } | SessionState::Qualifying { .. }
        );
        (was_in_session, now_in_session, prev_track, sta.track)
    }

    /// Returns the [`SessionKind`] that this `Rst` started, or `None` if the
    /// packet was a solicited reply that must not be treated as a fresh start.
    fn apply_rst(&self, rst: &Rst) -> Option<SessionKind> {
        // Solicited replies (reqi != 0) echo stale data and must not overwrite
        // the authoritative state set by Sta. Only unsolicited Rst packets
        // (reqi == 0, sent when a race genuinely starts) update state.
        if rst.reqi.0 != 0 {
            return None;
        }
        let mut g = self.inner.write();
        g.track = Some(rst.track);
        g.weather = Some(rst.weather);
        g.wind = Some(rst.wind);
        let kind = if rst.qualmins > 0 {
            g.session = SessionState::Qualifying {
                duration: Duration::from_secs(rst.qualmins as u64 * 60),
                flags: rst.flags,
            };
            SessionKind::Qualifying
        } else if let RaceLaps::Practice | RaceLaps::Untimed = rst.racelaps {
            // Practice / open / cruise: LFS reports no race in progress, so the
            // Sta-derived SessionState is left untouched - only the kind is
            // recorded so consumers can interpret Fin etc. correctly.
            if matches!(rst.racelaps, RaceLaps::Untimed) {
                SessionKind::Untimed
            } else {
                SessionKind::Practice
            }
        } else {
            g.session = SessionState::Racing {
                laps: rst.racelaps,
                flags: rst.flags,
            };
            SessionKind::Race
        };
        g.rst_count = g.rst_count.wrapping_add(1);
        Some(kind)
    }

    /// Returns `true` if the allowed-cars set changed.
    fn apply_allowed_cars(&self, cars: &PlcAllowedCarsSet) -> bool {
        let mut g = self.inner.write();
        if g.allowed_cars.as_ref() == Some(cars) {
            return false;
        }
        g.allowed_cars = Some(cars.clone());
        true
    }

    /// Returns `true` if the allowed-mods list changed.
    fn apply_allowed_mods(&self, mods: &[Vehicle]) -> bool {
        let mut g = self.inner.write();
        if g.allowed_mods.as_slice() == mods {
            return false;
        }
        g.allowed_mods = mods.to_vec();
        true
    }

    fn apply_version(&self, ver: &Ver) {
        let mut g = self.inner.write();
        g.version = Some(VersionInfo {
            product: ver.product.clone(),
            version: ver.version.clone(),
        });
    }

    fn apply_axi(&self, axi: &Axi) -> (Option<String>, Option<String>) {
        let mut g = self.inner.write();
        let prev = g.layout.clone();
        g.layout = axi.lname.clone();
        g.axi_count = g.axi_count.wrapping_add(1);
        (prev, axi.lname.clone())
    }

    fn apply_ism(&self, ism: &Ism) -> (MultiplayerState, MultiplayerState) {
        let mut g = self.inner.write();
        let prev = g.multiplayer.clone();
        // NOTE: If LFS is not in multiplayer mode, the host name in the ISM will be empty.
        g.multiplayer = match ism.hname.as_deref() {
            None | Some("") => MultiplayerState::Local,
            Some(name) => MultiplayerState::Multiplayer {
                host_name: name.to_owned(),
                is_host: ism.host,
            },
        };
        (prev, g.multiplayer.clone())
    }

    fn apply_tiny_axc(&self, tiny: &Tiny) -> Option<String> {
        if !matches!(tiny.subt, TinyType::Axc) {
            return None;
        }
        let mut g = self.inner.write();
        g.layout.take()
    }
}
