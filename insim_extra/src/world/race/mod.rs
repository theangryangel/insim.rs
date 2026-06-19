//! Race tracking: per-entrant data types, the [`RaceEvent`] output type, and
//! the [`RaceState`] accumulator driven by [`crate::world::World`].

mod entrant;
mod event;
mod state;

pub use entrant::{DriverRecord, EntrantId, EntrantState, FinishStatus, LapRecord, PitRecord};
pub use event::RaceEvent;
pub(crate) use state::RaceState;
