//! System-style handlers over an InSim packet/event stream.
//!
//! `insim_app` lets you write handlers as plain async functions with typed
//! "magic-extractor" parameters ([`Packet`], [`Event`], [`State`], [`Sender`]),
//! and add [`Extension`]s that combine queryable state with optional
//! `on_event` hooks. The dispatcher owns the connection directly - there is
//! no actor wrapper around the read half - and uses a single unbounded mpsc
//! back-channel for outbound packets and synthetic-event injection.
//!
//! The function-with-extractor surface is borrowed from axum, but the
//! runtime semantics are closer to Bevy: handlers are *systems* invoked on a
//! continuous stream of dispatches, [`Extension`]s are long-lived *resources*
//! shaped like Bevy's `Res<T>`, and synthetic events are a first-class
//! primitive. There is no ECS and no plugin abstraction - extensions are
//! flat. Handlers are gated implicitly by extractor type and explicitly via
//! [`HandlerExt::run_if`].
//!
//! ## Concurrency model
//!
//! - **Extensions' `on_event`** runs sequentially in registration order,
//!   *before* handlers. An extension may safely mutate its own `Arc`-backed
//!   internal state through interior mutability.
//! - **Handlers** for a given dispatch run **concurrently** via
//!   [`futures::stream::FuturesUnordered`]. They observe a shared `&State<S>`
//!   and may emit through the back-channel. **Any state two handlers may
//!   mutate concurrently must be atomic or lock-tolerant** - typically
//!   `Arc<AtomicX>`, `Arc<RwLock<…>>`, `Arc<Mutex<…>>`. Plain shared mutable
//!   state across handlers is unsound here.
//!
//! ## Emission semantics
//!
//! There is one emission API: [`Sender`]. From anywhere (extension, handler,
//! spawned task, UI thread) you call `sender.packet(p)` to write a packet
//! out, or `sender.event(e)` to inject a synthetic event. **Events fire in a
//! subsequent dispatch cycle, not the current one** - the back-channel is
//! drained by the main runtime loop between cycles.
//!
//! ## Shutdown
//!
//! Both [`ExtractCx`] and [`EventCx`] expose `shutdown()` / `is_shutdown()`.
//! Calling `cx.shutdown()` from a handler signals the runtime to exit at the
//! next select; the `Sender`'s back-channel and the framed connection are
//! dropped after the current cycle finishes.
//!
//! This crate is a proof of concept: it deliberately does not include scenes,
//! snapshots, compile-time tuple-recursive dispatch, or a tower/`Service`
//! integration. Those are planned follow-ups.
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

pub use app::{App, serve};
pub use error::AppError;
pub use event::{Dispatch, Startup};
pub use extensions::Extensions;
pub use extract::{Event, ExtractCx, FromContext, Packet, PacketVariant, Sender, State};
pub use game::{Game, GameInfo, RaceEnded, RaceStarted, TrackChanged};
pub use handler::Handler;
pub use middleware::{
    ChatParser, Connected, ConnectionDetails, ConnectionInfo, Disconnected, EventCx, Extension,
    PlayerInfo, PlayerJoined, PlayerLeft, PlayerTeleportedToPits, Presence, Renamed, TakingOver,
    VehicleSelected,
};
pub use run_if::{HandlerExt, Predicate, RunIf, always, and, in_state, never, not, or};
pub use spawned::{Spawned, spawned};
