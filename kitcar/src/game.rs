//! [`Handler`] and [`FromContext`] impls for [`insim_extra::game::Game`],
//! the named synthetic event structs, and the [`track_rotation`] orchestration
//! combinator.

use std::future::Future;

use insim::{
    WithRequestId,
    core::{game_version::GameVersion, track::Track, vehicle::Vehicle},
    identifiers::{ConnectionId, RequestId},
    insim::{PlcAllowedCarsSet, RaceLaps},
};
pub use insim_extra::game::{Game, GameEvent, GameInfo, SessionKind, SessionState, VersionInfo};
use insim_extra::util::mtc;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::{AppError, Dispatch, ExtractCx, FromContext, Handler, Sender, Startup};

/// Synthetic event emitted when an `Rst` starts a new race, qualifying,
/// practice or untimed session. This is the authoritative session-start signal.
#[derive(Debug, Clone)]
pub struct SessionStarted {
    /// Whether the session is a race, qualifying, practice or untimed.
    pub kind: SessionKind,
}

/// Synthetic event emitted when LFS returns to the entry/lobby screen
/// (`Sta` reports no race or qualifying in progress).
#[derive(Debug, Clone)]
pub struct SessionEnded;

/// Synthetic event emitted when the server's allowed-cars set changes.
#[derive(Debug, Clone)]
pub struct AllowedCarsChanged {
    /// The new allowed-cars set.
    pub cars: PlcAllowedCarsSet,
}

/// Synthetic event emitted when the server's allowed-mods list changes.
#[derive(Debug, Clone)]
pub struct AllowedModsChanged {
    /// The new allowed-mods list (empty means unrestricted).
    pub mods: Vec<Vehicle>,
}

/// Synthetic event emitted when version information is received.
#[derive(Debug, Clone)]
pub struct VersionReceived {
    /// Product name (e.g. `"S3"`).
    pub product: String,
    /// LFS game version.
    pub version: GameVersion,
}

/// Synthetic event emitted when the track changes.
#[derive(Debug, Clone)]
pub struct TrackChanged {
    /// Previously known track, if any.
    pub from: Option<Track>,
    /// Track now reported by LFS.
    pub to: Track,
}

/// Synthetic event emitted when the layout changes or is cleared.
#[derive(Debug, Clone)]
pub struct LayoutChanged {
    /// Previously known layout, if any.
    pub from: Option<String>,
    /// New layout, or `None` if cleared.
    pub to: Option<String>,
}

/// Synthetic event emitted when LFS joins or starts a multiplayer session.
#[derive(Debug, Clone)]
pub struct MultiplayerJoined {
    /// Host name of the server.
    pub host_name: String,
    /// `true` if this instance is the host.
    pub is_host: bool,
}

/// Synthetic event emitted when LFS leaves multiplayer (ISM with empty host name).
#[derive(Debug, Clone)]
pub struct MultiplayerLeft;

/// [`Game`] is its own extractor: register via
/// `app.handle(Stage::Pre, Game::new())` and any handler can take it by value.
impl<S> FromContext<S> for Game {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.lookup::<Game>()
    }
}

/// [`Handler`] impl delegates to [`Game::apply_packet`] and emits each
/// change as a typed synthetic event. Register at [`crate::Stage::Pre`] so the
/// game-state mirror is settled before Update-stage handlers read it.
///
/// On [`Startup`] it also sends `Tiny::Sst`, `Tiny::Axi`, `Tiny::Ism`,
/// `Tiny::Alc`, and `Tiny::Mal` to request the current game state, allowed
/// cars, and allowed mods (LFS does not send these automatically on connect).
/// `Tiny::Alc` and `Tiny::Mal` are also re-requested on each `SessionStarted`,
/// since a new session can change the allowed cars/mods.
impl<S: Send + Sync + 'static> Handler<(), S> for Game {
    fn call(self, cx: &ExtractCx<'_, S>) -> impl Future<Output = Result<(), AppError>> + Send {
        let events = if let Dispatch::Packet(p) = cx.dispatch {
            self.apply_packet(p)
        } else {
            vec![]
        };
        let startup = if let Dispatch::Synthetic(s) = cx.dispatch {
            s.downcast_ref::<Startup>().is_some()
        } else {
            false
        };
        // A new session can change the allowed cars/mods, so re-request them on
        // session start (mirrors LFS re-sending these in reply to a TINY on Rst).
        let session_started = events
            .iter()
            .any(|e| matches!(e, GameEvent::SessionStarted { .. }));
        let sender = cx.sender.clone();
        async move {
            if startup {
                for t in Game::STARTUP_REQUESTS {
                    let _ = sender.packet(t.clone().with_request_id(RequestId(1)));
                }
            } else if session_started {
                for t in Game::SESSION_REQUESTS {
                    let _ = sender.packet(t.clone().with_request_id(RequestId(1)));
                }
            }
            emit_game_events(events, &sender);
            Ok(())
        }
    }
}

fn emit_game_events(events: Vec<GameEvent>, sender: &Sender) {
    for event in events {
        let _ = match event {
            GameEvent::SessionStarted { kind } => sender.event(SessionStarted { kind }),
            GameEvent::SessionEnded => sender.event(SessionEnded),
            GameEvent::TrackChanged { from, to } => sender.event(TrackChanged { from, to }),
            GameEvent::LayoutChanged { from, to } => sender.event(LayoutChanged { from, to }),
            GameEvent::MultiplayerJoined { host_name, is_host } => {
                sender.event(MultiplayerJoined { host_name, is_host })
            },
            GameEvent::MultiplayerLeft => sender.event(MultiplayerLeft),
            GameEvent::AllowedCarsChanged { cars } => sender.event(AllowedCarsChanged { cars }),
            GameEvent::AllowedModsChanged { mods } => sender.event(AllowedModsChanged { mods }),
            GameEvent::VersionReceived { product, version } => {
                sender.event(VersionReceived { product, version })
            },
        };
    }
}

/// Full track-rotation combinator.
///
/// Returns `Some(())` once the new track is loaded and racing has started, or
/// `None` if `cancel` fires at any waiting step. Packet sends (`end`, `clear`,
/// `change_*`, `ax_clear`, `ax_load`) don't observe cancellation - if
/// cancellation fires mid-sequence the partial commands sent so far still went
/// out.
pub async fn track_rotation(
    game: &Game,
    track: Track,
    laps: RaceLaps,
    wind: u8,
    layout: Option<String>,
    cancel: CancellationToken,
    sender: &Sender,
) -> Option<()> {
    let current = game.get();
    info!(
        ?track,
        ?laps,
        wind,
        ?layout,
        current_track = ?current.current_track(),
        session = ?current.session(),
        "track_rotation: starting"
    );

    let in_game = matches!(
        current.session(),
        SessionState::Racing { .. } | SessionState::Qualifying { .. }
    );
    let track_differs = current.current_track() != Some(&track);

    let all = Some(ConnectionId::ALL);

    if in_game && track_differs {
        info!("track_rotation: in_game + track differs, sending /end, waiting for end");
        let _ = sender.packets(mtc("^3Setting up - changing track, please wait...", all));
        sender.packet(game.end()).ok()?;
        game.wait_for_end(cancel.clone()).await?;
        info!("track_rotation: end confirmed, sending /clear");
        sender.packet(game.clear()).ok()?;
    }

    if track_differs {
        info!(?track, "track_rotation: track differs, sending /track");
        sender.packet(game.change_track(track)).ok()?;
    } else {
        info!("track_rotation: same track, skipping /track");
    }

    info!(
        ?laps,
        wind, "track_rotation: sending /laps, /wind, /axclear"
    );
    sender.packet(game.change_laps(laps)).ok()?;
    sender.packet(game.change_wind(wind)).ok()?;
    sender.packet(game.ax_clear()).ok()?;

    if let Some(layout) = layout {
        info!(?layout, "track_rotation: sending /axload, waiting for Axi");
        let _ = sender.packets(mtc("^3Setting up - loading layout, please wait...", all));
        sender.packet(game.ax_load(layout)).ok()?;
        game.wait_for_any_axi(cancel.clone()).await?;
        info!("track_rotation: Axi received");
    }

    if in_game {
        info!("track_rotation: in_game, sending /restart");
        sender.packet(game.restart()).ok()?;
    }

    if track_differs {
        let _ = sender.packets(mtc("^2Setup complete - type ^3/ready^2 to start!", all));
    }

    info!("track_rotation: waiting for Rst");
    game.wait_for_any_rst(cancel).await?;
    info!("track_rotation: Rst received, done");
    Some(())
}
