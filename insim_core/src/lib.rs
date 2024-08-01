#![doc = include_str!("../README.md")]

pub mod duration;
pub mod game_version;
pub mod license;
pub mod point;
pub mod string;
pub mod track;
pub mod vehicle;
pub mod wind;

#[doc(hidden)]
pub use ::binrw;
