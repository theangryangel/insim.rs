//! # insim
//!
//! insim is a Rust library for working with the Racing Simulator Live For Speed.
//!
//! It's primary use case is to communicate with LFS via Insim, however it also provides additional
//! utilities for working with LFS as a whole.
//!
//! insim currently leans heavily on the Tokio ecosystem.
//!
//! ## Feature flags
//!
//! - `default`: Enables tcp, udp, relay
//! - `tcp`: Enable TCP support
//! - `udp`: Enable UDP support
//! - `relay`: Enable LFS World relay support
//! - `serde`: Enable serde support
//! - `game_data`: Pull in insim_game_data and re-export
//! - `pth`: Pull in insim_pth and re-export
//! - `smx`: Pull in insim_smx and re-export
//!
//! ## Examples
//!
//! See examples directory.

pub mod prelude;

pub mod codec;
pub mod connection;
pub mod error;
pub mod packets;
pub mod result;

#[cfg(feature = "udp")]
pub mod udp_stream;

#[cfg(feature = "websocket")]
pub mod websocket_stream;

#[doc(hidden)]
/// Rexport insim_core
pub use insim_core as core;

#[cfg(feature = "game_data")]
/// Report insim_game_data when game_data feature is enabled
pub use insim_game_data as game_data;

#[cfg(feature = "pth")]
/// Report insim_pth when pth feature is enabled
pub use insim_pth as pth;

#[cfg(feature = "smx")]
/// Report insim_smx when smx feature is enabled
pub use insim_smx as smx;
