//! Race tracking types. State accumulation logic lives in
//! [`crate::world::race`]; this module only re-exports the public data types.

mod entrant;
mod event;

pub use entrant::{DriverRecord, EntrantId, EntrantState, FinishStatus, LapRecord, PitRecord};
pub use event::RaceEvent;
