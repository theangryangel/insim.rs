//! Ready-made services to start next to your own.
//!
//! Each is an ordinary [`Service`](crate::kitcar::Service) you start on the
//! [`Host`](crate::kitcar::Host).
//!
//! - [`router`]: handlers for events and chat commands. Ideal for bots.
//! - [`race_director`]: sets up each combo in the lobby and rotates through them.
//! - [`penalty_clearer`]: clears each penalty after a fixed delay.

pub mod penalty_clearer;
pub mod race_director;
pub mod router;
