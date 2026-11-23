//! Ready-made services to start next to your own.
//!
//! Each is an ordinary [`Service`](crate::kitcar::Service) you start on the
//! [`Host`](crate::kitcar::Host).
//!
//! - [`router`]: handlers for events and chat commands. Ideal for bots.
//! - [`motd`]: a message of the day, shown to each player as they connect.
//! - [`penalty_clearer`]: clears each penalty after a fixed delay.

pub mod motd;
pub mod penalty_clearer;
pub mod router;
