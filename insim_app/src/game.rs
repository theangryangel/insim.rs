//! Game-state mirror plus host commands and a `track_rotation` combinator.
//!
//! Equivalent of `insim_extras::game`, adapted to `insim_app`'s
//! [`Extension`] / [`FromContext`] / captured-`Sender` pattern. Register with
//! `App::extension(Game::new(app.sender().clone()))` and pull into handlers
//! by value:
//!
//! ```ignore
//! async fn handler(game: Game) -> Result<(), AppError> {
//!     tracing::info!(?game.get().current_track(), "current track");
//!     game.end()?;
//!     Ok(())
//! }
//! ```
//!
//! On construction the extension sends a `Sst` request so the server pushes
//! initial `Sta` state; subsequent `Sta` packets keep the mirror current.
//!
//! Each `Sta` is also diffed against the previous mirror, and transition
//! events ([`RaceStarted`], [`RaceEnded`], [`TrackChanged`]) are emitted via
//! the back-channel so handlers can react to changes without polling.
//!
//! Commands are fire-and-forget (matching `Sender::packet` semantics).
//! Polling helpers (`wait_for_*`) take a [`CancellationToken`] and return
//! `Option<()>` - `None` means "cancelled before the predicate held".

use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use insim::{
    WithRequestId,
    core::{track::Track, wind::Wind},
    insim::{RaceInProgress, RaceLaps, StaFlags, TinyType},
};
use tokio_util::sync::CancellationToken;

use crate::{
    AppError,
    event::Dispatch,
    extract::{ExtractCx, FromContext, Sender},
    middleware::{EventCx, Extension},
    util::host_command,
};

/// Mirror of the relevant fields from an `Sta` packet.
#[derive(Debug, Default, Clone)]
pub struct GameInfo {
    track: Option<Track>,
    weather: Option<u8>,
    wind: Option<Wind>,
    racing: RaceInProgress,
    qualifying_duration: Duration,
    race_duration: RaceLaps,
    flags: StaFlags,
}

impl GameInfo {
    /// Currently selected track, if known.
    pub fn current_track(&self) -> Option<&Track> {
        self.track.as_ref()
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
}

/// Game-state extension. Cloneable; all clones see the same mirror.
#[derive(Clone)]
pub struct Game {
    inner: Arc<RwLock<GameInfo>>,
    sender: Sender,
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Game").finish_non_exhaustive()
    }
}

impl Game {
    /// Create a new game extension. Immediately sends a `Sst` request so the
    /// server pushes initial state; failure to send is silent (the
    /// back-channel will surface its own error elsewhere if it's closed).
    pub fn new(sender: Sender) -> Self {
        let this = Self {
            inner: Arc::new(RwLock::new(GameInfo::default())),
            sender,
        };
        let _ = this.sender.packet(TinyType::Sst.with_request_id(1));
        this
    }

    /// Snapshot of the current game state.
    pub fn get(&self) -> GameInfo {
        self.inner.read().expect("poison").clone()
    }

    // -----------------------------------------------------------------------
    // Host commands - fire-and-forget through the captured Sender.
    // -----------------------------------------------------------------------

    /// `/end` - finish the current race.
    pub fn end(&self) -> Result<(), AppError> {
        self.sender.packet(host_command("/end"))
    }

    /// `/track {track}` - load a different track. Use [`Self::track_rotation`]
    /// for the full end → wait → change → wait-for-racing flow.
    pub fn change_track(&self, track: Track) -> Result<(), AppError> {
        self.sender.packet(host_command(format!("/track {track}")))
    }

    /// Change race length. Maps onto `/laps`, `/hours`, or `/laps no`
    /// depending on the variant.
    pub fn change_laps(&self, laps: RaceLaps) -> Result<(), AppError> {
        let cmd = match laps {
            RaceLaps::Untimed => "/laps no".to_string(),
            RaceLaps::Hours(h) => format!("/hours {h}"),
            other => format!("/laps {}", Into::<u8>::into(other)),
        };
        self.sender.packet(host_command(cmd))
    }

    /// `/wind {wind}` - set wind strength (0..=2 typically).
    pub fn change_wind(&self, wind: u8) -> Result<(), AppError> {
        self.sender.packet(host_command(format!("/wind {wind}")))
    }

    /// `/axclear` - clear the autocross layout.
    pub fn ax_clear(&self) -> Result<(), AppError> {
        self.sender.packet(host_command("/axclear"))
    }

    /// `/axload {layout}` - load an autocross layout by name.
    pub fn ax_load(&self, layout: impl Into<String>) -> Result<(), AppError> {
        self.sender
            .packet(host_command(format!("/axload {}", layout.into())))
    }

    /// `/restart` - start a race.
    pub fn restart(&self) -> Result<(), AppError> {
        self.sender.packet(host_command("/restart"))
    }

    /// `/qualify` - start qualifying.
    pub fn qualify(&self) -> Result<(), AppError> {
        self.sender.packet(host_command("/qualify"))
    }

    /// `/reinit` - full restart, kicks all connections.
    pub fn reinit(&self) -> Result<(), AppError> {
        self.sender.packet(host_command("/reinit"))
    }

    /// `/weather {weather}` - set weather/lighting.
    pub fn change_weather(&self, weather: u8) -> Result<(), AppError> {
        self.sender
            .packet(host_command(format!("/weather {weather}")))
    }

    /// `/qual {minutes}` - set qualifying duration. `0` = no qualifying.
    pub fn change_qual(&self, minutes: u8) -> Result<(), AppError> {
        self.sender.packet(host_command(format!("/qual {minutes}")))
    }

    /// `/pit_all` - send every player to the pits.
    pub fn pit_all(&self) -> Result<(), AppError> {
        self.sender.packet(host_command("/pit_all"))
    }

    // -----------------------------------------------------------------------
    // Cancellable polling helpers.
    // -----------------------------------------------------------------------

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

    /// Full track-rotation combinator. Returns `Some(())` once the new
    /// track is loaded and racing has started, or `None` if `cancel` fires
    /// at any waiting step. Sync command sends (`end`, `change_*`,
    /// `ax_clear`, `ax_load`) don't observe cancel - if cancellation fires
    /// mid-sequence the partial commands sent so far still went out.
    pub async fn track_rotation(
        &self,
        track: Track,
        laps: RaceLaps,
        wind: u8,
        layout: Option<String>,
        cancel: CancellationToken,
    ) -> Option<()> {
        let current = self.get();
        if current.track.as_ref() != Some(&track) {
            self.end().ok()?;
            self.wait_for_end(cancel.clone()).await?;
            self.change_track(track).ok()?;
        }
        self.change_laps(laps).ok()?;
        self.change_wind(wind).ok()?;
        self.ax_clear().ok()?;
        if let Some(layout) = layout {
            self.ax_load(layout).ok()?;
        }
        self.wait_for_racing(cancel).await?;
        Some(())
    }
}

/// Synthetic event emitted by [`Game`] when an incoming `Sta` packet
/// transitions race-in-progress from non-racing → `Racing`.
#[derive(Debug, Clone)]
pub struct RaceStarted;

/// Synthetic event emitted by [`Game`] when an incoming `Sta` packet
/// transitions race-in-progress from `Racing` → non-racing.
#[derive(Debug, Clone)]
pub struct RaceEnded;

/// Synthetic event emitted by [`Game`] when an incoming `Sta` packet reports
/// a different track from the one previously mirrored. The first `Sta` after
/// `Game::new` (`from: None`) also produces a `TrackChanged`.
#[derive(Debug, Clone)]
pub struct TrackChanged {
    /// The previously known track, if any.
    pub from: Option<Track>,
    /// The track now reported by LFS.
    pub to: Track,
}

impl<S: Send + Sync + 'static> Extension<S> for Game {
    async fn on_event(&self, cx: &mut EventCx<'_, S>) {
        let Dispatch::Packet(insim::Packet::Sta(sta)) = cx.dispatch else {
            return;
        };

        // Mutate under the write lock; capture the values we need for the
        // transition diff. Drop the guard before emitting so any handler that
        // reacts to the synthetic events can freely re-enter `Game`.
        let (was_racing, now_racing, prev_track, new_track) = {
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
        };

        if !was_racing && now_racing {
            let _ = cx.sender.event(RaceStarted);
        }
        if was_racing && !now_racing {
            let _ = cx.sender.event(RaceEnded);
        }
        if prev_track != Some(new_track) {
            let _ = cx.sender.event(TrackChanged {
                from: prev_track,
                to: new_track,
            });
        }
    }
}

/// [`Game`] is its own extractor: register via [`crate::App::extension`] and
/// any handler can take it by value.
impl<S: Send + Sync + 'static> FromContext<S> for Game {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.extensions.get::<Game>()
    }
}
