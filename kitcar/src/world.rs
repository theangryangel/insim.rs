//! [`FromContext`] glue for [`insim_extra::world::World`] and the runtime's
//! mirror helpers.
//!
//! `World` is intrinsic to every [`crate::App`] (as core as the connection):
//! it is held as a field, folded by [`fold_packet`] before the causing packet
//! is dispatched, and extracted by value with a `world: World` parameter.
//!
//! ## Emission timing
//!
//! World-derived events are **not** delayed. [`fold_packet`] applies a packet
//! to the world and returns the resulting events as ready-to-dispatch
//! [`Dispatch`] values; the runtime dispatches them in the *same* loop
//! iteration, immediately after the packet itself. So a handler taking
//! `Event<Connected>` observes it in lockstep with the `Ncn` packet that caused
//! it - no inter-cycle lag. (User events emitted via [`crate::Sender::event`]
//! still ride the back-channel and fire in a subsequent cycle.)

use std::sync::Arc;

use insim::{WithRequestId, identifiers::RequestId};
pub use insim_extra::world::{World, WorldEvent};

use crate::{Dispatch, ExtractCx, FromContext, Sender};

/// [`World`] is its own extractor: every app embeds one, so this is infallible.
///
/// ## Example
///
/// ```ignore
/// async fn on_entrant_joined(
///     Event(event): Event<RaceEvent>,
///     world: World,
/// ) -> Result<(), AppError> {
///     if let RaceEvent::EntrantJoined { id, plid } = event {
///         // world.entrant(id), world.current_track(), world.session() available
///     }
///     Ok(())
/// }
/// ```
impl<S> FromContext<S> for World {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        Some(cx.world.clone())
    }
}

/// Send the combined startup requests ([`World::STARTUP_REQUESTS`]) once on
/// connect. LFS does not send these automatically, so the world would never
/// learn the existing connection/player/session state without them.
pub(crate) fn send_startup_requests(sender: &Sender) {
    for t in World::STARTUP_REQUESTS {
        let _ = sender.packet(t.clone().with_request_id(RequestId(1)));
    }
}

/// Fold one wire packet into [`World`] and return its downstream events as
/// ready-to-dispatch synthetics.
///
/// [`World::apply_packet`] runs all three state mirrors in the correct order in
/// a single call. If applying the packet started a session, the combined
/// [`World::SESSION_REQUESTS`] are re-sent. Each resulting [`WorldEvent`] is
/// wrapped as its concrete payload struct (see [`world_event_dispatch`]) so the
/// caller can dispatch it through the normal `Event<T>` machinery.
pub(crate) fn fold_packet(world: &World, packet: &insim::Packet, sender: &Sender) -> Vec<Dispatch> {
    let events = world.apply_packet(packet);
    let session_started = events
        .iter()
        .any(|e| matches!(e, WorldEvent::SessionStarted(_)));
    if session_started {
        for t in World::SESSION_REQUESTS {
            let _ = sender.packet(t.clone().with_request_id(RequestId(1)));
        }
    }
    events.into_iter().map(world_event_dispatch).collect()
}

/// Wrap a [`WorldEvent`] as a [`Dispatch::Synthetic`] carrying its concrete
/// payload struct (not the `WorldEvent` enum), so handlers taking
/// `Event<Connected>`, `Event<RaceEvent>`, etc. match on it directly.
fn world_event_dispatch(event: WorldEvent) -> Dispatch {
    fn synthetic<T: std::any::Any + Send + Sync>(payload: T) -> Dispatch {
        Dispatch::Synthetic(Arc::new(payload))
    }
    match event {
        WorldEvent::Connected(e) => synthetic(e),
        WorldEvent::Disconnected(e) => synthetic(e),
        WorldEvent::ConnectionDetails(e) => synthetic(e),
        WorldEvent::VehicleSelected(e) => synthetic(e),
        WorldEvent::Renamed(e) => synthetic(e),
        WorldEvent::PlayerJoined(e) => synthetic(e),
        WorldEvent::PlayerLeft(e) => synthetic(e),
        WorldEvent::TakingOver(e) => synthetic(e),
        WorldEvent::PlayerTeleportedToPits(e) => synthetic(e),
        WorldEvent::SessionStarted(e) => synthetic(e),
        WorldEvent::SessionEnded(e) => synthetic(e),
        WorldEvent::TrackChanged(e) => synthetic(e),
        WorldEvent::LayoutChanged(e) => synthetic(e),
        WorldEvent::MultiplayerJoined(e) => synthetic(e),
        WorldEvent::MultiplayerLeft(e) => synthetic(e),
        WorldEvent::AllowedCarsChanged(e) => synthetic(e),
        WorldEvent::AllowedModsChanged(e) => synthetic(e),
        WorldEvent::VersionReceived(e) => synthetic(e),
        WorldEvent::Race(e) => synthetic(e),
    }
}
