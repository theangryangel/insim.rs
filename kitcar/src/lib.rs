//! Kitcar - intended to help you make simple single server mini-games quickly using the `insim` crate.
//! By itself it does nothing.

#[allow(missing_docs)]
pub mod ui;

pub mod combos;
pub mod game;
pub mod leaderboard;
pub mod presence;
pub mod time;

pub use kitcar_macros::service;

/// Reactive background service. Basically something you want to blindly run something on a packet
/// query
pub trait Service {
    /// Broadcast capacity to use
    const BROADCAST_CAPACITY: usize = 32;

    /// Spawn as a background task, returning a handle for easy querying
    fn spawn(insim: insim::builder::SpawnedHandle);
}
