//! Kitcar - intended to help you make simple single server mini-games quickly using the `insim` crate.
//! By itself it does nothing.

pub mod context;
pub mod engine;
pub mod lease;
pub mod timers;
pub mod world;

pub use context::{ConnectionInfo, Context, PlayerInfo};
pub use engine::Engine;
pub use timers::Timer;
pub use world::{Chassis, Workshop};
