//! Synthetic event structs for game-state changes and the [`track_rotation`]
//! orchestration combinator.

use insim::{
    core::{game_version::GameVersion, track::Track, vehicle::Vehicle},
    identifiers::ConnectionId,
    insim::{PlcAllowedCarsSet, RaceLaps},
};
pub use insim_extra::game::{GameInfo, SessionKind, SessionState, VersionInfo};
use insim_extra::{util::mtc, world::World};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::Sender;

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

/// Full track-rotation combinator.
///
/// Returns `Some(())` once the new track is loaded and racing has started, or
/// `None` if `cancel` fires at any waiting step. Packet sends (`end`, `clear`,
/// `change_*`, `ax_clear`, `ax_load`) don't observe cancellation - if
/// cancellation fires mid-sequence the partial commands sent so far still went
/// out.
pub async fn track_rotation(
    world: &World,
    track: Track,
    laps: RaceLaps,
    wind: u8,
    layout: Option<String>,
    cancel: CancellationToken,
    sender: &Sender,
) -> Option<()> {
    let current_track = world.current_track();
    let session = world.session();
    info!(
        ?track,
        ?laps,
        wind,
        ?layout,
        current_track = ?current_track,
        session = ?session,
        "track_rotation: starting"
    );

    let in_game = matches!(
        session,
        SessionState::Racing { .. } | SessionState::Qualifying { .. }
    );
    let track_differs = current_track != Some(track);

    let all = Some(ConnectionId::ALL);

    if in_game && track_differs {
        info!("track_rotation: in_game + track differs, sending /end, waiting for end");
        let _ = sender.packets(mtc("^3Setting up - changing track, please wait...", all));
        sender.packet(world.end()).ok()?;
        world.wait_for_end(cancel.clone()).await?;
        info!("track_rotation: end confirmed, sending /clear");
        sender.packet(world.clear()).ok()?;
    }

    if track_differs {
        info!(?track, "track_rotation: track differs, sending /track");
        sender.packet(world.change_track(track)).ok()?;
    } else {
        info!("track_rotation: same track, skipping /track");
    }

    info!(
        ?laps,
        wind, "track_rotation: sending /laps, /wind, /axclear"
    );
    sender.packet(world.change_laps(laps)).ok()?;
    sender.packet(world.change_wind(wind)).ok()?;
    sender.packet(world.ax_clear()).ok()?;

    if let Some(layout) = layout {
        info!(?layout, "track_rotation: sending /axload, waiting for Axi");
        let _ = sender.packets(mtc("^3Setting up - loading layout, please wait...", all));
        sender.packet(world.ax_load(layout)).ok()?;
        world.wait_for_any_axi(cancel.clone()).await?;
        info!("track_rotation: Axi received");
    }

    if in_game {
        info!("track_rotation: in_game, sending /restart");
        sender.packet(world.restart()).ok()?;
    }

    if track_differs {
        let _ = sender.packets(mtc("^2Setup complete - type ^3/ready^2 to start!", all));
    }

    info!("track_rotation: waiting for Rst");
    world.wait_for_any_rst(cancel).await?;
    info!("track_rotation: Rst received, done");
    Some(())
}
