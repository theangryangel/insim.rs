//! [`Handler`] and [`FromContext`] impls for [`insim_extra::world::World`].

use std::future::Future;

use insim::{WithRequestId, identifiers::RequestId};
pub use insim_extra::world::{World, WorldEvent};

use crate::{
    AppError, Dispatch, ExtractCx, FromContext, Handler, Sender, Startup,
    game::{
        AllowedCarsChanged, AllowedModsChanged, LayoutChanged, MultiplayerJoined, MultiplayerLeft,
        SessionEnded, SessionStarted, TrackChanged, VersionReceived,
    },
    presence::{
        Connected, ConnectionDetails, Disconnected, PlayerJoined, PlayerLeft,
        PlayerTeleportedToPits, Renamed, TakingOver, VehicleSelected,
    },
};

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
/// inter-cycle lag), then emits the same individual synthetic event types the
/// existing split setup produced, so Update-stage handlers using
/// `Event<Connected>`, `Event<RaceEvent>`, etc. are unchanged.
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
///         // world.race(), world.current_track(), world.session() all available
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
            .any(|e| matches!(e, WorldEvent::SessionStarted { .. }));
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

fn emit_world_events(events: Vec<WorldEvent>, sender: &Sender) {
    for event in events {
        let _ = match event {
            WorldEvent::Connected(info) => sender.event(Connected(info)),
            WorldEvent::Disconnected { ucid, info } => sender.event(Disconnected { ucid, info }),
            WorldEvent::ConnectionDetails(info) => sender.event(ConnectionDetails(info)),
            WorldEvent::VehicleSelected { ucid, vehicle } => {
                sender.event(VehicleSelected { ucid, vehicle })
            },
            WorldEvent::Renamed {
                ucid,
                uname,
                new_pname,
            } => sender.event(Renamed {
                ucid,
                uname,
                new_pname,
            }),
            WorldEvent::PlayerJoined(p) => sender.event(PlayerJoined(p)),
            WorldEvent::PlayerLeft(p) => sender.event(PlayerLeft(p)),
            WorldEvent::TakingOver { before, after } => sender.event(TakingOver { before, after }),
            WorldEvent::PlayerTeleportedToPits(p) => sender.event(PlayerTeleportedToPits(p)),
            WorldEvent::SessionStarted { kind } => sender.event(SessionStarted { kind }),
            WorldEvent::SessionEnded => sender.event(SessionEnded),
            WorldEvent::TrackChanged { from, to } => sender.event(TrackChanged { from, to }),
            WorldEvent::LayoutChanged { from, to } => sender.event(LayoutChanged { from, to }),
            WorldEvent::MultiplayerJoined { host_name, is_host } => {
                sender.event(MultiplayerJoined { host_name, is_host })
            },
            WorldEvent::MultiplayerLeft => sender.event(MultiplayerLeft),
            WorldEvent::AllowedCarsChanged { cars } => sender.event(AllowedCarsChanged { cars }),
            WorldEvent::AllowedModsChanged { mods } => sender.event(AllowedModsChanged { mods }),
            WorldEvent::VersionReceived { product, version } => {
                sender.event(VersionReceived { product, version })
            },
            WorldEvent::Race(re) => sender.event(re),
        };
    }
}
