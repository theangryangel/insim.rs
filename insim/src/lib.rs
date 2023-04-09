//! insim.rs is a Rust library for working with the Racing Simulator Live For Speed.
//!
//! It's primary use case is to communicate with LFS via Insim, however it also provides additional
//! utilities for working with LFS as a whole.

pub mod prelude;

pub mod client;
pub mod codec;
pub mod error;
pub mod packets;
pub mod result;

#[cfg(feature = "udp")]
pub mod udp_stream;

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
