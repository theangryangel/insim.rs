//! insim.rs is a Rust library for working with the Racing Simulator Live For Speed.
//!
//! It's primary use case is to communicate with LFS via Insim, however it also provides additional
//! utilities for working with LFS as a whole.
//!
//! The library provides both [lower level](protocol) and an optional [higher level](framework) APIs for working with Insim, however at
//! this time it only supports TCP. As a result Outsim is not supported at this time.

pub mod conversion;
pub mod error;
#[cfg(feature = "framework")]
pub mod framework;
pub mod protocol;
pub mod string;
pub mod track;
pub mod vehicle;
