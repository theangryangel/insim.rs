//! Per-entrant state types stored by [`super::RaceTracker`].

use std::time::Duration;

use insim::{
    identifiers::{ConnectionId, PlayerId},
    insim::{PenaltyInfo, PitStopWorkFlags, RaceConfirmFlags},
};

/// Stable identity for one race entry (one `Npl` → leave lifecycle).
///
/// Never reused within a [`super::RaceTracker`] instance, even if LFS
/// reuses the same [`PlayerId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntrantId(pub(super) u64);

impl std::fmt::Display for EntrantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntrantId({})", self.0)
    }
}

/// Per-lap record stored in [`EntrantState`].
#[derive(Debug, Clone)]
pub struct LapRecord {
    /// Lap number (1-indexed).
    pub lap: u16,
    /// Lap time.
    pub time: Duration,
    /// Split times recorded during this lap (up to 3).
    pub splits: Vec<Duration>,
    /// Pit stop count at the time this lap completed.
    pub numstops: u8,
    /// Penalty state at the time this lap completed.
    pub penalty: PenaltyInfo,
}

/// Per-pit-stop record stored in [`EntrantState`].
#[derive(Debug, Clone)]
pub struct PitRecord {
    /// Cumulative stop number.
    pub stop_number: u8,
    /// Lap on which the stop occurred.
    pub lap: u16,
    /// Work carried out.
    pub work: PitStopWorkFlags,
    /// Total time in the pit box. `None` if the `Psf` packet never arrived.
    pub stop_time: Option<Duration>,
}

/// Finish status for an [`EntrantState`].
#[derive(Debug, Clone, Default)]
pub enum FinishStatus {
    /// Still racing.
    #[default]
    Racing,
    /// Crossed the finish line (provisional result from `Fin`).
    Finished {
        /// Total race time.
        ttime: Duration,
        /// Best lap time.
        btime: Duration,
        /// Number of pit stops.
        numstops: u8,
        /// Confirmation flags (penalties, DQ, etc.).
        confirm: RaceConfirmFlags,
        /// Classified position. Populated when the `Res` packet arrives.
        result_num: Option<u8>,
    },
    /// Left the track without finishing while the race was active.
    Dnf,
}

/// Record of one driver who controlled a car during a race entry.
#[derive(Debug, Clone)]
pub struct DriverRecord {
    /// Connection ID at the time this driver took the wheel.
    pub ucid: ConnectionId,
    /// Display name at the time this driver took the wheel.
    pub pname: String,
    /// LFS.net username, if connection details were available.
    pub uname: Option<String>,
    /// Lap number when this driver started driving (0 = from the start).
    pub from_lap: u16,
}

/// Per-entrant race state stored by [`super::RaceTracker`].
#[derive(Debug, Clone)]
pub struct EntrantState {
    /// Stable synthetic identifier.
    pub id: EntrantId,
    /// [`PlayerId`] assigned by LFS at join time. May be reused by other
    /// entrants in future races.
    pub plid: PlayerId,
    /// Laps completed so far (corrected by [`lap_offset`](Self::lap_offset)).
    pub laps_done: u16,
    /// Added to the raw LFS lap counter to produce the true running total.
    ///
    /// Non-zero after [`apply_player_rejoined`](super::RaceTracker::apply_player_rejoined)
    /// or [`apply_telepit_resume`](super::RaceTracker::apply_telepit_resume), where LFS
    /// resets its internal lap counter to 1 but we want to preserve continuity.
    pub lap_offset: u16,
    /// Fastest lap time recorded.
    pub best_lap: Option<Duration>,
    /// Lap number on which [`best_lap`](Self::best_lap) was set.
    pub best_lap_num: Option<u16>,
    /// Completed lap records, in order.
    pub laps: Vec<LapRecord>,
    /// Split times for the lap currently in progress.
    pub current_splits: Vec<Duration>,
    /// Completed pit stops.
    pub pit_stops: Vec<PitRecord>,
    /// Finish status.
    pub status: FinishStatus,
    /// Driver history. The current driver is the last entry.
    pub drivers: Vec<DriverRecord>,
    /// Pending pit stop: `Pit` received, waiting for matching `Psf`.
    pub(super) pending_pit: Option<PitRecord>,
    /// Active penalty.
    pub penalty: PenaltyInfo,
}
