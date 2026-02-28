//! Clockwork Carnage — shared library for event and challenge binaries.

#![allow(missing_docs, missing_debug_implementations)]

pub mod chat;
pub mod components;
pub mod leaderboard;
pub mod scenes;
pub mod spawn_control;

/// Minimum number of connections required to start (host + 1 player).
pub const MIN_PLAYERS: usize = 2;
