//! Per-entrant race state: laps, pit stops, drivers and result.

use std::time::Duration;

use insim::{
    identifiers::{ConnectionId, PlayerId},
    insim::{PenaltyInfo, PitStopWorkFlags, RaceConfirmFlags},
};

/// Stable identity for one race entry, from its `Npl` until it leaves.
///
/// Survives driver swaps, and is never reused within a [`crate::world::World`]
/// even when LFS reuses the [`PlayerId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntrantId(pub(crate) u64);

impl std::fmt::Display for EntrantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntrantId({})", self.0)
    }
}

/// One completed lap in an [`EntrantState`].
#[derive(Debug, Clone)]
pub struct LapRecord {
    /// Lap number, from 1.
    pub lap: u16,
    /// Lap time.
    pub time: Duration,
    /// Splits crossed this lap (up to 3).
    pub splits: Vec<Duration>,
    /// Pit stops so far.
    pub numstops: u8,
    /// Penalty at the end of the lap.
    pub penalty: PenaltyInfo,
}

/// One pit stop in an [`EntrantState`].
#[derive(Debug, Clone)]
pub struct PitRecord {
    /// Cumulative stop number.
    pub stop_number: u8,
    /// Lap the stop happened on.
    pub lap: u16,
    /// Work done.
    pub work: PitStopWorkFlags,
    /// Time in the pit box. `None` if the `Psf` never arrived.
    pub stop_time: Option<Duration>,
}

/// How an [`EntrantState`]'s race ended, if it has.
#[derive(Debug, Clone, Default)]
pub enum FinishStatus {
    /// Still racing.
    #[default]
    Racing,
    /// Crossed the line. Provisional from `Fin` until `Res` confirms it.
    Finished {
        /// Total race time.
        ttime: Duration,
        /// Best lap.
        btime: Duration,
        /// Pit stops.
        numstops: u8,
        /// Confirmation flags (penalties, DQ, etc.).
        confirm: RaceConfirmFlags,
        /// Classified position, once `Res` arrives.
        result_num: Option<u8>,
    },
    /// Left the track mid-race without finishing.
    Dnf,
}

/// A driver who took the wheel during a race entry.
#[derive(Debug, Clone)]
pub struct DriverRecord {
    /// Their connection when they took over.
    pub ucid: ConnectionId,
    /// Their display name when they took over.
    pub pname: String,
    /// LFS.net username, if known.
    pub uname: Option<String>,
    /// Lap they took over on (0 = from the start).
    pub from_lap: u16,
}

/// Race state for one entrant, kept by [`crate::world::World`].
#[derive(Debug, Clone)]
pub struct EntrantState {
    /// Stable identifier.
    pub id: EntrantId,
    /// Assigned by LFS on join. Later entrants may reuse it.
    pub plid: PlayerId,
    /// Laps completed, including [`lap_offset`](Self::lap_offset).
    pub laps_done: u16,
    /// Added to LFS's lap counter to give the true total.
    ///
    /// Non-zero after a rejoin (see [`crate::world::World::with_rejoin`]). LFS
    /// sends a fresh `Npl` and restarts its counter at 1, but the entrant's laps
    /// carry on.
    pub lap_offset: u16,
    /// Fastest lap.
    pub best_lap: Option<Duration>,
    /// Lap [`best_lap`](Self::best_lap) was set on.
    pub best_lap_num: Option<u16>,
    /// Completed laps, in order.
    pub laps: Vec<LapRecord>,
    /// Splits for the lap in progress.
    pub current_splits: Vec<Duration>,
    /// Completed pit stops.
    pub pit_stops: Vec<PitRecord>,
    /// Finish status.
    pub status: FinishStatus,
    /// Everyone who has driven this entry. The current driver is last.
    pub drivers: Vec<DriverRecord>,
    /// Grid slot (from 1), from `Reo`. `None` until a grid order covers this
    /// entrant.
    pub grid_position: Option<u8>,
    /// `Pit` received, waiting for its `Psf`.
    pub(crate) pending_pit: Option<PitRecord>,
    /// Current penalty.
    pub penalty: PenaltyInfo,
}
