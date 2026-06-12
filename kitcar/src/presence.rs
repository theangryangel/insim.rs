//! [`Handler`] and [`FromContext`] impls for [`insim_extra::presence::Presence`],
//! plus the synthetic event structs that `kitcar` dispatch uses.

use std::future::Future;

use insim::WithRequestId;
pub use insim_extra::presence::{ConnectionInfo, PlayerInfo, Presence, PresenceEvent};

use crate::{AppError, Dispatch, ExtractCx, FromContext, Handler, Sender, Startup};

/// Synthetic event emitted when a connection joins.
#[derive(Debug, Clone)]
pub struct Connected(pub ConnectionInfo);

/// Synthetic event emitted when a connection leaves.
#[derive(Debug, Clone)]
pub struct Disconnected {
    /// The connection that left.
    pub ucid: insim::identifiers::ConnectionId,
    /// Last known info for the connection.
    pub info: Option<ConnectionInfo>,
}

/// Synthetic event emitted when a connection changes their display name.
#[derive(Debug, Clone)]
pub struct Renamed {
    /// Connection ID.
    pub ucid: insim::identifiers::ConnectionId,
    /// LFS.net username (stable).
    pub uname: String,
    /// New display name.
    pub new_pname: String,
}

/// Synthetic event emitted when extra connection details arrive via `Nci`.
#[derive(Debug, Clone)]
pub struct ConnectionDetails(pub ConnectionInfo);

/// Synthetic event emitted when a connection selects a vehicle in the garage.
#[derive(Debug, Clone)]
pub struct VehicleSelected {
    /// Connection that selected the vehicle.
    pub ucid: insim::identifiers::ConnectionId,
    /// The selected vehicle.
    pub vehicle: insim::core::vehicle::Vehicle,
}

/// Synthetic event emitted when a player joins the track.
#[derive(Debug, Clone)]
pub struct PlayerJoined(pub PlayerInfo);

/// Synthetic event emitted when a player leaves the track.
#[derive(Debug, Clone)]
pub struct PlayerLeft(pub PlayerInfo);

/// Synthetic event emitted when a player's controlling connection changes.
#[derive(Debug, Clone)]
pub struct TakingOver {
    /// The player before the swap.
    pub before: PlayerInfo,
    /// The player after the swap.
    pub after: PlayerInfo,
}

/// Synthetic event emitted when a player tele-pits (Shift+P).
#[derive(Debug, Clone)]
pub struct PlayerTeleportedToPits(pub PlayerInfo);

/// [`Presence`] is its own extractor: register via
/// `app.handle(Stage::Pre, Presence::new())` and any handler can take it by value.
impl<S> FromContext<S> for Presence {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.lookup::<Presence>()
    }
}

/// [`Handler`] impl delegates to [`Presence::apply_packet`] and emits each
/// change as a typed synthetic event. Register at [`crate::Stage::Pre`] so the
/// connection / player maps are settled before any Update-stage handler reads them.
///
/// On [`Startup`] it also sends `Tiny::Ncn` and `Tiny::Npl` to request the
/// existing connection and player lists from LFS (they are not sent automatically
/// on connect).
impl<S: Send + Sync + 'static> Handler<(), S> for Presence {
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
        let sender = cx.sender.clone();
        async move {
            if startup {
                for t in Presence::STARTUP_REQUESTS {
                    let _ = sender.packet(t.clone().with_request_id(1));
                }
            }
            emit_presence_events(events, &sender);
            Ok(())
        }
    }
}

fn emit_presence_events(events: Vec<PresenceEvent>, sender: &Sender) {
    for event in events {
        let _ = match event {
            PresenceEvent::Connected(info) => sender.event(Connected(info)),
            PresenceEvent::Disconnected { ucid, info } => sender.event(Disconnected { ucid, info }),
            PresenceEvent::ConnectionDetails(info) => sender.event(ConnectionDetails(info)),
            PresenceEvent::VehicleSelected { ucid, vehicle } => {
                sender.event(VehicleSelected { ucid, vehicle })
            },
            PresenceEvent::Renamed {
                ucid,
                uname,
                new_pname,
            } => sender.event(Renamed {
                ucid,
                uname,
                new_pname,
            }),
            PresenceEvent::PlayerJoined(p) => sender.event(PlayerJoined(p)),
            PresenceEvent::PlayerLeft(p) => sender.event(PlayerLeft(p)),
            PresenceEvent::TakingOver { before, after } => {
                sender.event(TakingOver { before, after })
            },
            PresenceEvent::PlayerTeleportedToPits(p) => sender.event(PlayerTeleportedToPits(p)),
        };
    }
}
