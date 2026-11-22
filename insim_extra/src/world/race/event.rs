//! [`RaceEvent`]: what race tracking reports.

use std::time::Duration;

use insim::{
    identifiers::{ConnectionId, PlayerId},
    insim::{PenaltyInfo, PenaltyReason, RaceConfirmFlags},
};

use super::entrant::{EntrantId, LapRecord, PitRecord};

/// What race tracking in [`crate::world`] reports.
///
/// Session starts arrive as [`crate::world::WorldEvent::SessionStarted`], not
/// here.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum RaceEvent {
    /// An entrant joined the track.
    EntrantJoined {
        /// The entrant.
        id: EntrantId,
        /// LFS player ID on joining.
        plid: PlayerId,
    },
    /// An entrant completed a lap.
    LapCompleted {
        /// The entrant.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// The lap.
        record: LapRecord,
    },
    /// An entrant crossed a split.
    SplitCompleted {
        /// The entrant.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// Split index (1 to 3).
        split: u8,
        /// Split time.
        time: Duration,
    },
    /// A pit stop finished: both `Pit` and `Psf` arrived.
    PitStopComplete {
        /// The entrant.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// The pit stop.
        record: PitRecord,
    },
    /// A penalty changed.
    PenaltyChanged {
        /// The entrant.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// Previous penalty.
        oldpen: PenaltyInfo,
        /// New penalty.
        newpen: PenaltyInfo,
        /// Why it changed.
        reason: PenaltyReason,
    },
    /// A driver swap. The entrant and player ID stay the same.
    DriverSwap {
        /// The entrant.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// The incoming driver's connection.
        new_ucid: ConnectionId,
    },
    /// An entrant crossed the finish line. Provisional until
    /// [`ResultConfirmed`](Self::ResultConfirmed).
    Finished {
        /// The entrant.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// Total race time.
        ttime: Duration,
        /// Best lap.
        btime: Duration,
        /// Confirmation flags.
        confirm: RaceConfirmFlags,
    },
    /// LFS confirmed a result (`Res`).
    ResultConfirmed {
        /// The entrant.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// Classified position, from 0.
        result_num: u8,
        /// How many results are classified.
        num_results: u8,
    },
    /// An entrant left the track mid-race without finishing.
    Dnf {
        /// The entrant.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
    },
    /// A disconnected driver came back and was matched to their old
    /// [`super::EntrantState`] by LFS.net username.
    ///
    /// They keep their [`EntrantId`] and lap history. A
    /// [`lap_offset`](super::EntrantState::lap_offset) makes later
    /// [`RaceEvent::LapCompleted`] records carry the true lap total, not LFS's
    /// counter, which restarts at 1.
    ///
    /// Only emitted by a world built with [`crate::world::World::with_rejoin`].
    EntrantRejoined {
        /// The entrant, same as before the disconnect.
        id: EntrantId,
        /// New LFS player ID.
        plid: PlayerId,
    },
    /// An entrant beat their own best lap.
    ///
    /// Follows the [`RaceEvent::LapCompleted`] that set it. Their first lap
    /// always counts.
    PersonalBest {
        /// The entrant.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// Lap it was set on.
        lap: u16,
        /// The new best.
        time: Duration,
        /// The previous best, if any.
        previous: Option<Duration>,
    },
    /// A new fastest lap of the session, across all entrants.
    ///
    /// Follows the [`RaceEvent::LapCompleted`] that set it. The session's first
    /// lap always counts.
    FastestLap {
        /// The entrant.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// Lap it was set on, including any
        /// [`lap_offset`](super::EntrantState::lap_offset).
        lap: u16,
        /// Lap time.
        time: Duration,
    },
    /// An entrant teleported to the pits (Shift+P, `Plp`).
    ///
    /// The lap in progress is dropped, but the lap total stays: LFS doesn't
    /// reset its counter on a telepit.
    TeleportedToPits {
        /// The entrant.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
    },
}
