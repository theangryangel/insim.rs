//! insim.rs is a Rust library for working with the Racing Simulator Live For Speed.
//!
//! It's primary use case is to communicate with LFS via Insim, however it also provides additional
//! utilities for working with LFS as a whole.

pub mod prelude;

pub mod codec;
pub mod config;
pub mod error;
pub mod packets;
pub mod transport;

#[doc(hidden)]
/// Rexport insim_core
pub use insim_core as core;
