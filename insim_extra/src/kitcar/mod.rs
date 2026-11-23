//! Run games and services on an InSim connection.
//!
//! A [`Host`] owns one connection and keeps a [`World`] in step with the
//! server. It runs any number of [`Service`]s: a game, the idle screen between
//! events, or something long-lived. Start and stop them without reconnecting.
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
//! - **Ordered.** Each packet updates the [`World`] first, then reaches every
//!   service, followed by the world events it caused. Inputs arrive one at a
//!   time, so state needs no locks.
//! - **No I/O in updates.** `update` is synchronous and returns [`Effects`]:
//!   packets, keyed timers and tasks for slow work, whose results come back as
//!   messages. Services stay deterministic, so a test is just `update` with an
//!   input and a [`Context`].
//! - **Services side by side.** Behaviour that needs nothing from a game, such
//!   as [`PenaltyClearer`], runs as its own service. The game follows the
//!   session through the [`World`], so it doesn't matter who started a race.
//!   Behaviour that needs a service's state is a struct field with its own
//!   update method: combine with [`Effects::and`], lift messages with
//!   [`Effects::map`].
//! - **One director.** A [`RaceDirector`] runs the races themselves: lobby
//!   setup, restarts and the combo rotation. It isn't a service. The host owns
//!   at most one, given with [`Host::start_director`], runs it before every
//!   service, and lets every service read it with [`Context::director`].
//!   Read it to show what's coming: races left, the restart countdown, the
//!   next track.
//!
//! # Routers
//!
//! A bot that mostly reacts to events and commands doesn't need a whole
//! `update`. [`Router`] is a [`Service`] built from handlers, each picked by
//! the type it takes:
//!
//! ```ignore
//! Router::new(Bot::default())
//!     .on(joined)          // fn(&mut Bot, &Connected, &Context) -> Effects<()>
//!     .on(uco)             // fn(&mut Bot, &Uco, &Context) -> ...
//!     .commands(command)   // fn(&mut Bot, Caller, Cmd, &Context) -> ...
//! ```
//!
//! Routers suit things that react. Games and other state machines, where the
//! order of checks matters, are clearer as a hand-written `update`.
//!
//! # Multiplayer only
//!
//! kitcar targets multiplayer servers. The local connection (UCID 0) is the
//! server, never a player: it gets no views, and [`World::connections`] leaves
//! it out. Local and single-player tools can drive a [`crate::ui::Canvas`] or
//! [`crate::ui::Screen`] directly.
//!
//! [`World`]: crate::world::World
//! [`Router`]: services::router::Router
//! [`RaceDirector`]: director::RaceDirector
//! [`PenaltyClearer`]: services::penalty_clearer::PenaltyClearer
//! [`World::connections`]: crate::world::World::connections

pub mod director;
pub mod effects;
pub mod error;
pub mod host;
pub mod input;
pub mod service;
pub mod services;
mod timers;

pub use effects::Effects;
pub use host::Host;
pub use service::{Context, Input, Service};
