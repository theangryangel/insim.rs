//! Race tracking: each entrant's laps, splits, pit stops and result.
//!
//! During race and qualifying sessions a [`World`](crate::world::World) keeps an
//! [`EntrantState`] per car and reports changes as [`RaceEvent`]s. Entrants are
//! keyed by [`EntrantId`], which survives driver swaps and, unlike a
//! `PlayerId`, is never reused.

mod entrant;
mod event;
mod state;

#[cfg(test)]
mod tests;

pub use entrant::{DriverRecord, EntrantId, EntrantState, FinishStatus, LapRecord, PitRecord};
pub use event::RaceEvent;
pub(crate) use state::RaceState;
