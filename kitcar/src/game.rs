//! [`Handler`] and [`FromContext`] impls for [`insim_extra::game::Game`],
//! the named synthetic event structs, and the [`track_rotation`] orchestration
//! combinator.

use std::future::Future;

use insim::{
    core::track::Track,
    insim::{RaceLaps, TinyType},
};
pub use insim_extra::game::{Game, GameEvent, GameInfo};
use tokio_util::sync::CancellationToken;

use crate::{AppError, Dispatch, ExtractCx, FromContext, Handler, Sender, Startup};

/// Synthetic event emitted when the race transitions to racing.
#[derive(Debug, Clone)]
pub struct RaceStarted;

/// Synthetic event emitted when the race transitions from racing to non-racing.
#[derive(Debug, Clone)]
pub struct RaceEnded;

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

/// [`Handler`] impl delegates to [`Game::apply_events`] and emits each
/// change as a typed synthetic event. Register at [`crate::Stage::Pre`] so the
/// game-state mirror is settled before Update-stage handlers read it.
///
/// On [`Startup`] it also sends `Tiny::Axi` to request the current layout
/// (LFS does not send this automatically on connect).
impl<S: Send + Sync + 'static> Handler<(), S> for Game {
    fn call(self, cx: &ExtractCx<'_, S>) -> impl Future<Output = Result<(), AppError>> + Send {
        let events = if let Dispatch::Packet(p) = cx.dispatch {
            self.apply_events(p)
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
                let _ = sender.packet(TinyType::Sst);
                let _ = sender.packet(TinyType::Axi);
                let _ = sender.packet(TinyType::Ism);
            }
            emit_game_events(events, &sender);
            Ok(())
        }
    }
}

fn emit_game_events(events: Vec<GameEvent>, sender: &Sender) {
    for event in events {
        let _ = match event {
            GameEvent::RaceStarted => sender.event(RaceStarted),
            GameEvent::RaceEnded => sender.event(RaceEnded),
            GameEvent::TrackChanged { from, to } => sender.event(TrackChanged { from, to }),
            GameEvent::LayoutChanged { from, to } => sender.event(LayoutChanged { from, to }),
            GameEvent::MultiplayerJoined { host_name, is_host } => {
                sender.event(MultiplayerJoined { host_name, is_host })
            },
            GameEvent::MultiplayerLeft => sender.event(MultiplayerLeft),
        };
    }
}

/// Full track-rotation combinator.
///
/// Returns `Some(())` once the new track is loaded and racing has started, or
/// `None` if `cancel` fires at any waiting step. Packet sends (`end`,
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
    if current.current_track() != Some(&track) {
        sender.packet(game.end()).ok()?;
        game.wait_for_end(cancel.clone()).await?;
        sender.packet(game.change_track(track)).ok()?;
    }
    sender.packet(game.change_laps(laps)).ok()?;
    sender.packet(game.change_wind(wind)).ok()?;
    sender.packet(game.ax_clear()).ok()?;
    if let Some(layout) = layout {
        sender.packet(game.ax_load(layout.clone())).ok()?;
        game.wait_for_layout(layout, cancel.clone()).await?;
    }
    game.wait_for_racing(cancel).await?;
    Some(())
}
