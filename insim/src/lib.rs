//! insim.rs is a Rust library for working with the Racing Simulator Live For Speed.
//!
//! It's primary use case is to communicate with LFS via Insim, however it also provides additional
//! utilities for working with LFS as a whole.
//!
//! The library provides both [lower level](protocol) and an optional [higher level](client) APIs for working with Insim, however at
//! this time it only supports TCP. As a result Outsim is not supported at this time.

#[cfg(feature = "client")]
pub mod client;
pub mod error;
pub mod protocol;

#[doc(hidden)]
/// Rexport insim_core
pub use insim_core as core;
