//! [`Handler`] and [`FromContext`] impls for
//! [`insim_extra::race::RaceTracker`].

use std::future::Future;

use insim::{WithRequestId, identifiers::RequestId, insim::TinyType};
pub use insim_extra::race::{
    DriverRecord, EntrantId, EntrantState, FinishStatus, LapRecord, PitRecord, RaceEvent,
    RaceTracker,
};

use crate::{
    AppError, Dispatch, ExtractCx, FromContext, Handler,
    game::{SessionEnded, SessionKind, SessionStarted},
    presence::{Connected, Disconnected, PlayerJoined, PlayerLeft, Renamed, TakingOver},
};

/// [`RaceTracker`] is its own extractor: register via
/// `app.handle(Stage::Pre, RaceTracker::new())` and any handler can take it
/// by value.
impl<S> FromContext<S> for RaceTracker {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.lookup::<RaceTracker>()
    }
}

/// [`Handler`] impl for [`RaceTracker`].
///
/// Handles both raw timing packets and synthetic events from [`crate::Presence`]
/// and [`crate::Game`].
///
/// **Registration order matters:** register at [`crate::Stage::Pre`] *after*
/// both `Presence` and `Game`. Synthetic events from those handlers
/// (`PlayerJoined`, `SessionStarted`, etc.) are deferred by one dispatch cycle,
/// so `RaceTracker` receives them in the cycle after they were emitted.
///
/// **Packets handled directly** (same cycle as arrival):
/// `Lap`, `Spx`, `Fin`, `Res`, `Pit`, `Psf`, `Pen`, `Plp`, `Reo`
///
/// **Synthetic events handled** (one cycle after Presence/Game emit them):
/// `Connected`, `Disconnected`, `Renamed`, `PlayerJoined`, `PlayerLeft`,
/// `TakingOver`, `SessionStarted`, `SessionEnded`
///
/// On `SessionStarted` the tracker clears, then re-requests the player list
/// (`Tiny::Npl`) and - for race/qualifying sessions - the grid order
/// (`Tiny::Reo`) so the entrant list is rebuilt for the new session.
///
/// Each resulting [`RaceEvent`] is emitted as a synthetic via
/// [`Sender::event`] for Update-stage handlers to consume via
/// `Event<RaceEvent>`.
///
/// ## Example
///
/// ```ignore
/// app.handle(Stage::Pre, Presence::new())
///    .handle(Stage::Pre, Game::new())
///    .handle(Stage::Pre, RaceTracker::new())  // after Presence and Game
///    .handle(Stage::Update, on_race_event)
///
/// async fn on_race_event(
///     Event(event): Event<RaceEvent>,
///     race: RaceTracker,
/// ) -> Result<(), AppError> {
///     match event {
///         RaceEvent::LapCompleted { id, plid, record } => { /* ... */ },
///         RaceEvent::Finished { id, plid, .. } => { /* ... */ },
///         _ => {},
///     }
///     Ok(())
/// }
/// ```
impl<S: Send + Sync + 'static> Handler<(), S> for RaceTracker {
    fn call(self, cx: &ExtractCx<'_, S>) -> impl Future<Output = Result<(), AppError>> + Send {
        // Set when a session just started so the freshly-cleared tracker can be
        // repopulated by re-requesting the player list (and, for sessions with a
        // grid, the grid order).
        let mut resync: Option<SessionKind> = None;
        let events = match cx.dispatch {
            Dispatch::Packet(p) => self.apply_packet(p),
            Dispatch::Synthetic(s) => {
                if let Some(Connected(info)) = s.downcast_ref::<Connected>() {
                    self.apply_connected(info);
                    vec![]
                } else if let Some(ev) = s.downcast_ref::<Disconnected>() {
                    self.apply_disconnected(ev.ucid);
                    vec![]
                } else if let Some(ev) = s.downcast_ref::<Renamed>() {
                    self.apply_renamed(ev.ucid, &ev.uname, &ev.new_pname);
                    vec![]
                } else if let Some(PlayerJoined(info)) = s.downcast_ref::<PlayerJoined>() {
                    self.apply_player_joined(info)
                } else if let Some(PlayerLeft(info)) = s.downcast_ref::<PlayerLeft>() {
                    self.apply_player_left(info)
                } else if let Some(ev) = s.downcast_ref::<TakingOver>() {
                    self.apply_taking_over(&ev.before, &ev.after)
                } else if let Some(ev) = s.downcast_ref::<SessionStarted>() {
                    resync = Some(ev.kind);
                    self.apply_session_started(ev.kind)
                } else if s.downcast_ref::<SessionEnded>().is_some() {
                    self.apply_race_ended();
                    vec![]
                } else {
                    vec![]
                }
            },
        };
        let sender = cx.sender.clone();
        async move {
            if let Some(kind) = resync {
                // LFS does not push these on session start, so request them to
                // rebuild the entrant list that `apply_session_started` cleared.
                let _ = sender.packet(TinyType::Npl.with_request_id(RequestId(1)));
                if matches!(kind, SessionKind::Race | SessionKind::Qualifying) {
                    let _ = sender.packet(TinyType::Reo.with_request_id(RequestId(1)));
                }
            }
            for event in events {
                let _ = sender.event(event);
            }
            Ok(())
        }
    }
}
