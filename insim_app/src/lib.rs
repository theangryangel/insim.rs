//! Axum-style handler runtime over an InSim connection.
//!
//! `insim_app` lets you write handlers as plain async functions with typed
//! "magic-extractor" parameters ([`Packet`], [`Event`], [`State`], [`Sender`]),
//! attach them to a [`Router`], add [`Middleware`] that can emit synthetic
//! events, and spawn long-running tasks via [`SpawnedHandler`]. The dispatcher
//! owns the connection directly — there is no actor wrapper around the read
//! half — and uses a single mpsc back-channel for outbound packets and
//! synthetic-event injection.
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
mod handler;
mod middleware;
pub mod util;

#[cfg(test)]
mod tests;

pub use app::{App, serve};
pub use error::AppError;
pub use event::{Dispatch, Emitter, Startup};
pub use extensions::Extensions;
pub use extract::{Event, ExtractCx, FromContext, Packet, PacketVariant, Sender, State};
pub use handler::Handler;
pub use middleware::{
    ChatParser, ConnectionInfo, Connected, Disconnected, EventCx, Extension, Presence,
};
