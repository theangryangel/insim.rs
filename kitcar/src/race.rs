//! [`Handler`] and [`FromContext`] impls for
//! [`insim_extra::race::RaceTracker`].

use std::future::Future;

pub use insim_extra::race::{
    DriverRecord, EntrantId, EntrantState, FinishStatus, LapRecord, PitRecord, RaceEvent,
    RaceTracker,
};

use crate::{
    AppError, Dispatch, ExtractCx, FromContext, Handler,
    game::{RaceEnded, RaceStarted},
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
/// (`PlayerJoined`, `RaceStarted`, etc.) are deferred by one dispatch cycle,
/// so `RaceTracker` receives them in the cycle after they were emitted.
///
/// **Packets handled directly** (same cycle as arrival):
/// `Lap`, `Spx`, `Fin`, `Res`, `Pit`, `Psf`, `Pen`, `Plp`
///
/// **Synthetic events handled** (one cycle after Presence/Game emit them):
/// `Connected`, `Disconnected`, `Renamed`, `PlayerJoined`, `PlayerLeft`,
/// `TakingOver`, `RaceStarted`, `RaceEnded`
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
                } else if s.downcast_ref::<RaceStarted>().is_some() {
                    self.apply_race_started()
                } else if s.downcast_ref::<RaceEnded>().is_some() {
                    self.apply_race_ended();
                    vec![]
                } else {
                    vec![]
                }
            },
        };
        // FIXME: We need Synthetic events. Right now we're just sending out RaceEvent.
        let sender = cx.sender.clone();
        async move {
            for event in events {
                let _ = sender.event(event);
            }
            Ok(())
        }
    }
}
