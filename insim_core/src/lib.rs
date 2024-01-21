#![deny(unused_crate_dependencies)]
#![doc = include_str!("../README.md")]

pub mod duration;
pub mod identifiers;
pub mod license;
pub mod point;
pub mod racelaps;
pub mod string;
pub mod track;
pub mod vehicle;
pub mod wind;

#[doc(hidden)]
pub use ::binrw;
