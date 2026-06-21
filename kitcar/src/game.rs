//! Game-state event types (re-exported from [`insim_extra::world`]) and the
//! [`track_rotation`] orchestration combinator.

use insim::{core::track::Track, identifiers::ConnectionId, insim::RaceLaps};
pub use insim_extra::world::{
    AllowedCarsChanged, AllowedModsChanged, GameInfo, LayoutChanged, MultiplayerJoined,
    MultiplayerLeft, SessionEnded, SessionKind, SessionStarted, TrackChanged, VersionInfo,
    VersionReceived,
};
use insim_extra::{util::mtc, world::World};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::Sender;

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
    let current_track = world.track();
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
        Some(SessionKind::Race { .. }) | Some(SessionKind::Qualifying { .. })
    );
    let track_differs = current_track != Some(track);

    let all = Some(ConnectionId::ALL);

    if in_game && track_differs {
        info!("track_rotation: in_game + track differs, sending /end, waiting for end");
        let _ = sender.packets(mtc("^3Setting up - changing track, please wait...", all));
        sender.packet(world.end()).ok()?;
        world
            .wait_until(cancel.clone(), |w| w.session().is_none().then_some(()))
            .await?;
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
        let axi_before = world.axi_count();
        sender.packet(world.ax_load(layout)).ok()?;
        world
            .wait_until(cancel.clone(), |w| {
                (w.axi_count() != axi_before).then_some(())
            })
            .await?;
        info!("track_rotation: Axi received");
    }

    // Snapshot before sending /restart so an Rst that lands before we start
    // waiting is still observed.
    let rst_before = world.rst_count();

    if in_game {
        info!("track_rotation: in_game, sending /restart");
        sender.packet(world.restart()).ok()?;
    }

    if track_differs {
        let _ = sender.packets(mtc("^2Setup complete - type ^3/ready^2 to start!", all));
    }

    info!("track_rotation: waiting for Rst");
    world
        .wait_until(cancel, |w| (w.rst_count() != rst_before).then_some(()))
        .await?;
    info!("track_rotation: Rst received, done");
    Some(())
}
