//! System-style handlers over an InSim packet/event stream.
//!
//! `insim_app` lets you write handlers as plain async functions with typed
//! "magic-extractor" parameters ([`Packet`], [`Event`], [`Res`], [`Sender`]),
//! and register long-lived typed values as **resources**. Resources are pure
//! typed data (`TypeId`-keyed); behaviour lives in handlers. To bundle a
//! resource together with its handlers, implement [`Installable`] and
//! register via [`App::install`].
//!
//! The function-with-extractor surface is borrowed from axum, but the runtime
//! semantics are closer to Bevy: handlers are *systems* invoked on a
//! continuous stream of dispatches, resources are long-lived state shaped
//! like Bevy's `Res<T>`, and synthetic events are a first-class primitive.
//! There is no ECS, no plugin trait — installables are flat ensembles.
//! Handlers are gated implicitly by extractor type and explicitly via
//! [`HandlerExt::run_if`].
//!
//! ## Concurrency model
//!
//! Handlers for a given dispatch run **concurrently** via
//! [`futures::stream::FuturesUnordered`]. They observe shared resources via
//! `&Extensions` and may emit through the back-channel. **Any resource two
//! handlers mutate concurrently must be atomic or lock-tolerant** — typically
//! `Arc<AtomicX>`, `Arc<RwLock<…>>`, `Arc<Mutex<…>>`.
//!
//! ## Emission semantics
//!
//! One emission API: [`Sender`]. From anywhere (handler, spawned task, UI
//! thread) `sender.packet(p)` writes a packet out, `sender.event(e)` injects
//! a synthetic event. **Events fire in a subsequent dispatch cycle, not the
//! current one** — the back-channel is drained by the main runtime loop
//! between cycles.
//!
//! ## Shutdown
//!
//! [`ExtractCx`] exposes `shutdown()` / `is_shutdown()`. Calling
//! `cx.shutdown()` from a handler signals the runtime to exit at the next
//! select; the `Sender`'s back-channel and the framed connection are dropped
//! after the current cycle finishes.
//!
//! See `examples/smoke.rs` for an end-to-end example.

mod app;
mod error;
mod event;
mod extensions;
mod extract;
mod game;
mod handler;
mod middleware;
pub mod run_if;
mod spawned;
pub mod time;
#[allow(missing_docs)]
pub mod ui;
pub mod util;

#[cfg(test)]
mod tests;

pub use app::{App, Installable, serve};
pub use error::AppError;
pub use event::{Dispatch, Startup};
pub use extensions::Extensions;
pub use extract::{Event, ExtractCx, FromContext, Packet, Res, Sender};
pub use game::{Game, GameInfo, RaceEnded, RaceStarted, TrackChanged, game_on_sta};
pub use handler::Handler;
pub use middleware::{
    Connected, ConnectionDetails, ConnectionInfo, Disconnected, PlayerInfo, PlayerJoined,
    PlayerLeft, PlayerTeleportedToPits, Presence, Renamed, TakingOver, VehicleSelected,
    chat_parser,
};
pub use run_if::{HandlerExt, Predicate, RunIf, always, and, in_state, never, not, or};
pub use spawned::{Spawned, spawned};
