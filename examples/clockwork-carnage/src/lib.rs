//! Clockwork Carnage — shared library for event and challenge binaries.

#![allow(missing_docs, missing_debug_implementations)]

pub mod challenge;
pub mod components;
pub mod db;
pub mod event;
pub mod setup_track;
pub mod spawn_control;

pub type ChatError = kitcar::chat::RuntimeError;

/// Minimum number of connections required to start (host + 1 player).
pub const MIN_PLAYERS: usize = 2;
