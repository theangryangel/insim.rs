//! Kitcar - intended to help you make simple single server mini-games quickly using the `insim` crate.
//! By itself it does nothing.

#[allow(missing_docs)]
pub mod ui;

pub mod combos;
pub mod game;
pub mod leaderboard;
pub mod presence;
pub mod time;
pub mod utils;

/// Reactive background service. Basically a state without a handle.
pub trait Service {
    /// Broadcast capacity to use
    const BROADCAST_CAPACITY: usize = 32;

    /// Spawn as a background task, returning a handle for easy querying
    fn spawn(insim: insim::builder::SpawnedHandle);
}

/// Handler / Tracker
pub trait State {
    /// Lightweight, cloneable handle to communicate with spawned State
    type H: Clone;

    /// Broadcast capacity to use
    const BROADCAST_CAPACITY: usize = 32;

    /// Update an instance from an insim Packet
    fn update(&mut self, packet: &insim::Packet);

    /// Spawn as a background task, returning a handle for easy querying
    fn spawn(insim: insim::builder::SpawnedHandle) -> Self::H;
}
