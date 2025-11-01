//! Phases
mod game;
mod idle;
mod lobby;
mod track_rotation;

pub(crate) use game::{round, victory};
pub(crate) use idle::idle;
pub(crate) use lobby::lobby;
pub(crate) use track_rotation::track_rotation;
