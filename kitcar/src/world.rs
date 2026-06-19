//! [`Handler`] and [`FromContext`] impls for [`insim_extra::world::World`].

use std::future::Future;

use insim::{WithRequestId, identifiers::RequestId};
pub use insim_extra::world::{World, WorldEvent};

use crate::{AppError, Dispatch, ExtractCx, FromContext, Handler, Sender, Startup};

/// [`World`] is its own extractor: register via
/// `app.handle(Stage::Pre, World::new())` and any handler can take it by value.
impl<S> FromContext<S> for World {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.lookup::<World>()
    }
}

/// [`Handler`] impl for [`World`].
///
/// Replaces the three separate `Presence`, `Game`, and `RaceTracker` Pre-stage
/// handlers with one aggregate. On each packet, [`World::apply_packet`] runs
/// all three state mirrors in the correct order in a single call (no
/// inter-cycle lag), then emits each event's individual payload struct (see
/// [`emit_world_events`]) so Update-stage handlers using `Event<Connected>`,
/// `Event<RaceEvent>`, etc. fire as before.
///
/// On [`Startup`] it sends the combined startup requests from
/// [`World::STARTUP_REQUESTS`]. On [`WorldEvent::SessionStarted`] it sends
/// [`World::SESSION_REQUESTS`].
///
/// ## Example
///
/// ```ignore
/// app.handle(Stage::Pre, World::new())
///    .handle(Stage::Update, on_entrant_joined)
///
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
impl<S: Send + Sync + 'static> Handler<(), S> for World {
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
        let session_started = events
            .iter()
            .any(|e| matches!(e, WorldEvent::SessionStarted(_)));
        let sender = cx.sender.clone();
        async move {
            if startup {
                for t in World::STARTUP_REQUESTS {
                    let _ = sender.packet(t.clone().with_request_id(RequestId(1)));
                }
            } else if session_started {
                for t in World::SESSION_REQUESTS {
                    let _ = sender.packet(t.clone().with_request_id(RequestId(1)));
                }
            }
            emit_world_events(events, &sender);
            Ok(())
        }
    }
}

/// Fan each [`WorldEvent`] out as its concrete payload type so the `Event<T>`
/// extractor can dispatch on it. Each arm injects the wrapped payload struct
/// (not the `WorldEvent`), so handlers taking `Event<Connected>`,
/// `Event<RaceEvent>`, etc. fire as before.
fn emit_world_events(events: Vec<WorldEvent>, sender: &Sender) {
    for event in events {
        let _ = match event {
            WorldEvent::Connected(e) => sender.event(e),
            WorldEvent::Disconnected(e) => sender.event(e),
            WorldEvent::ConnectionDetails(e) => sender.event(e),
            WorldEvent::VehicleSelected(e) => sender.event(e),
            WorldEvent::Renamed(e) => sender.event(e),
            WorldEvent::PlayerJoined(e) => sender.event(e),
            WorldEvent::PlayerLeft(e) => sender.event(e),
            WorldEvent::TakingOver(e) => sender.event(e),
            WorldEvent::PlayerTeleportedToPits(e) => sender.event(e),
            WorldEvent::SessionStarted(e) => sender.event(e),
            WorldEvent::SessionEnded(e) => sender.event(e),
            WorldEvent::TrackChanged(e) => sender.event(e),
            WorldEvent::LayoutChanged(e) => sender.event(e),
            WorldEvent::MultiplayerJoined(e) => sender.event(e),
            WorldEvent::MultiplayerLeft(e) => sender.event(e),
            WorldEvent::AllowedCarsChanged(e) => sender.event(e),
            WorldEvent::AllowedModsChanged(e) => sender.event(e),
            WorldEvent::VersionReceived(e) => sender.event(e),
            WorldEvent::Race(e) => sender.event(e),
        };
    }
}
