//! Run games and services on an InSim connection.
//!
//! A [`Host`] owns one connection for as long as it lasts and keeps a [`World`]
//! in step with the server. It runs any number of [`Service`]s: a game such as a
//! time trial, the idle screen between events, or a long-lived service. Services can
//! be started and stopped at any time without reconnecting.
//!
//! A service is one update function over its own state:
//!
//! ```ignore
//! enum Msg { Boom(PlayerId) }
//!
//! impl Service for Bomb {
//!     type Msg = Msg;
//!     fn update(&mut self, input: Input<Msg>, cx: &Context) -> Effects<Msg> {
//!         match input {
//!             Input::Packet(Packet::Uco(uco)) => self.checkpoint(&uco, cx),
//!             Input::Msg(Msg::Boom(plid)) => self.explode(plid, cx),
//!             _ => Effects::none(),
//!         }
//!     }
//! }
//! ```
//!
//! - **Ordered.** The host folds each packet into the [`World`] first, then gives
//!   it to every service, followed by the world events it caused. Inputs arrive one
//!   at a time, so state needs no locks.
//! - **No I/O in updates.** `update` is synchronous and returns [`Effects`]:
//!   packets to send, timers (keyed, so they can be replaced or cancelled), and
//!   tasks for slow work such as database queries, whose results come back as
//!   messages. This keeps services deterministic and easy to test: call `update`
//!   with an input and a [`Context`], and inspect the effects.
//! - **Components, not handlers.** Reusable behaviour such as [`Lifecycle`] and
//!   [`PenaltyClearer`] is a plain struct held as a field, with its own update
//!   method. Combine effects with [`Effects::and`], and lift a component's
//!   messages into your own with [`Effects::map`].
//!
//! # Multiplayer only
//!
//! kitcar targets multiplayer servers. The local connection (UCID 0) is always
//! the server, never a player: views are never drawn for it, and
//! [`World::connections`] leaves it out. Local and single-player tools can drive
//! an [`insim_extra::ui::Canvas`] or [`insim_extra::ui::Screen`] directly.

mod effects;
mod error;
mod host;
mod lifecycle;
mod penalty_clearer;
mod service;
mod timers;

pub use effects::{Effects, TimerKey};
pub use error::AppError;
pub use host::{Host, ServiceHandle};
pub use insim_extra::{
    chat::{ChatEvent, ChatParser},
    ui,
    util::{host_command, mtc},
    world::{self, World, WorldEvent, WorldReady},
};
pub use lifecycle::{Layout, Lifecycle, Phase, Setup, Transition};
pub use penalty_clearer::{ClearPenalty, PenaltyClearer};
pub use service::{Context, Input, Service};
