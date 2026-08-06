//! System-style handlers over an InSim packet/event stream.
//!
//! `kitcar` lets you write handlers as plain async functions with typed
//! "magic-extractor" parameters ([`Packet`], [`Event`], [`Sender`],
//! [`State`]). Handlers can also be implemented manually on a
//! struct that owns state; see [`Handler`].
//!
//! The function-with-extractor surface is borrowed from axum. Handlers are
//! gated implicitly by extractor type and explicitly via
//! [`HandlerExt::run_if`].
//!
//! ## State vs handlers
//!
//! Two state shapes:
//!
//! - **[`State<S>`]** is the bot's *primary* state - one value, type-checked
//!   at the [`App`] level. Build with [`App::with_state(value)`](App::with_state)
//!   and extract via [`State<S>`]. The framework holds the value in an
//!   clone of `S`; applications choose any interior mutability they need.
//!   Bots without a primary state use
//!   [`App::new`] (which yields `App<()>`) and never touch `State`.
//! - **Handlers** may own private state. Each registered handler is uniquely
//!   owned by the runtime and called through `&mut self`; handlers are not a
//!   service registry and cannot be extracted by other handlers.
//!
//! Rule of thumb: if it's *the* thing the bot revolves around, use
//! `with_state`. State private to one registration can live directly in that
//! handler.
//!
//! ## Dispatch model
//!
//! The intrinsic [`World`] mirror is folded first, then handlers run
//! sequentially in registration order. Register state-maintaining handlers
//! before consumers that must observe their effects.
//!
//! Handlers can emit synthetic events via [`Sender::event`]; those events
//! fire in a *subsequent* cycle, where the Pre-first ordering applies
//! again.
//!
//! Handlers that need independent concurrency can spawn background work and
//! communicate back through [`Sender`]. Kitcar does not add a lock around
//! [`State<S>`].
//!
//! ## Emission semantics
//!
//! One emission API: [`Sender`]. From anywhere (handler, spawned task, UI
//! thread) `sender.packet(p)` writes a single packet out, `sender.packets(iter)` writes multiple, `sender.event(e)` injects
//! a synthetic event. **Events fire in a subsequent dispatch cycle, not the
//! current one** - the back-channel is drained by the main runtime loop
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
mod chat;
mod error;
mod game;
mod penalty_clearer;
mod presence;
#[allow(missing_docs)]
pub mod ui;
mod world;

pub use app::{
    App, Installable,
    event::{Dispatch, Shutdown, Startup},
    extract::{Event, ExtractCx, FromContext, Packet, State},
    handler::Handler,
    run_if,
    run_if::{HandlerExt, RunIf},
    runtime::{Sender, run, run_connection},
};
pub use chat::{ChatEvent, ChatParser};
pub use error::AppError;
pub use game::{
    AllowedCarsChanged, AllowedModsChanged, GameInfo, LayoutChanged, LobbyEntered, LobbyNotReady,
    LobbyReady, MultiplayerJoined, MultiplayerLeft, SessionEnded, SessionKind, SessionStarted,
    TrackChanged, VersionInfo, VersionReceived,
};
pub use insim_extra::{
    util::{host_command, mtc},
    world::{DriverRecord, EntrantId, EntrantState, FinishStatus, LapRecord, PitRecord, RaceEvent},
};
pub use penalty_clearer::PenaltyClearer;
pub use presence::{
    Connected, ConnectionDetails, ConnectionInfo, Disconnected, PlayerInfo, PlayerJoined,
    PlayerLeft, PlayerTeleportedToPits, Renamed, TakingOver, VehicleSelected,
};
pub use world::{World, WorldEvent};
