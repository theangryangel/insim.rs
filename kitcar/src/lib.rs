//! System-style handlers over an InSim packet/event stream.
//!
//! `kitcar` lets you write handlers as plain async functions with typed
//! "magic-extractor" parameters ([`Packet`], [`Event`], [`Sender`],
//! [`State`], [`Svc`]). Handlers can also be implemented manually on a
//! struct that owns state; see [`Handler`].
//!
//! The function-with-extractor surface is borrowed from axum. Handlers are
//! gated implicitly by extractor type and explicitly via
//! [`HandlerExt::run_if`].
//!
//! ## State vs handlers
//!
//! Two storage shapes:
//!
//! - **[`State<S>`]** is the bot's *primary* state - one value, type-checked
//!   at the [`App`] level. Build with [`App::with_state(value)`](App::with_state)
//!   and extract via [`State<S>`]. The framework holds the value in an
//!   `Arc<RwLock<S>>` internally; users write plain `S` and call `.read()` /
//!   `.write()` on the extractor. Bots without a primary state use
//!   [`App::new`] (which yields `App<()>`) and never touch `State`.
//! - **Handlers** are everything else. Every handler registered via
//!   [`App::handle`] sits in a `TypeId`-keyed map per stage; that map
//!   serves both for dispatch ordering and for typed extraction by other
//!   handlers. A stateful handler is just a struct that manually impls
//!   [`Handler`] (`Presence`, `Game`, `Ui` all follow this pattern, as
//!   does any user-side stateful handler). Passive data types can also
//!   impl `Handler<(), S>` with the trait's default no-op `call` and be
//!   registered for cross-cutting extraction.
//!
//! Rule of thumb: if it's *the* thing the bot revolves around, use
//! `with_state`. Everything else is a handler - register at the
//! appropriate [`Stage`].
//!
//! ## Dispatch model
//!
//! Every dispatch runs in two phases:
//!
//! 1. **[`Stage::Pre`]** - handlers run *sequentially* in registration
//!    order. Deciders that the concurrent Update handlers gate on (e.g.
//!    [`RoundManager`]) live here so their effects are settled first. (The
//!    intrinsic [`World`] mirror is folded by the runtime *ahead of both
//!    stages*, so every handler already observes settled world state.)
//! 2. **[`Stage::Update`]** - handlers run *concurrently* via
//!    [`futures::stream::FuturesUnordered`]. Most game logic lives here.
//!
//! Handlers can emit synthetic events via [`Sender::event`]; those events
//! fire in a *subsequent* cycle, where the Pre-first ordering applies
//! again.
//!
//! ## Concurrency model
//!
//! Update-stage handlers run **concurrently**. They observe shared handler
//! values via the per-stage maps and may emit through the back-channel.
//! **Any state two handlers mutate concurrently must be atomic or
//! lock-tolerant** - typically `Arc<AtomicX>`, `Arc<RwLock<…>>`,
//! `Arc<Mutex<…>>`. The [`State<S>`] extractor wraps an `Arc<RwLock<S>>`
//! for you.
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
mod round;
#[allow(missing_docs)]
pub mod ui;
mod world;

pub use app::{
    App, Installable, Stage,
    event::{Dispatch, Shutdown, Startup},
    extract::{Event, ExtractCx, FromContext, Packet, State, Svc},
    handler::Handler,
    run_if,
    run_if::{HandlerExt, RunIf},
    runtime::{Sender, run},
};
pub use chat::{ChatEvent, ChatParser};
pub use error::AppError;
pub use game::{
    AllowedCarsChanged, AllowedModsChanged, GameInfo, LayoutChanged, MultiplayerJoined,
    MultiplayerLeft, SessionEnded, SessionKind, SessionStarted, TrackChanged, VersionInfo,
    VersionReceived, track_rotation,
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
pub use round::{
    RoundEndReason, RoundEnded, RoundManager, RoundPhase, RoundPolicy, RoundSpec, RoundStarted,
};
pub use world::{World, WorldEvent};
