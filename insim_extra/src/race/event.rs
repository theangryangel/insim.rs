//! [`RaceEvent`] - output events emitted by race-tracking functions.

use std::time::Duration;

use insim::{
    identifiers::{ConnectionId, PlayerId},
    insim::{PenaltyInfo, PenaltyReason, RaceConfirmFlags},
};

use super::entrant::{EntrantId, LapRecord, PitRecord};
use crate::game::SessionKind;

/// Events emitted by race-tracking `apply_*` functions in [`crate::world`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum RaceEvent {
    /// A race or qualifying session started (`Rst`) and all prior state was
    /// cleared.
    SessionStarted {
        /// Whether the new session is a race or qualifying.
        kind: SessionKind,
    },
    /// A new entrant joined the track.
    EntrantJoined {
        /// Stable entrant identifier.
        id: EntrantId,
        /// LFS player ID at the time of joining.
        plid: PlayerId,
    },
    /// A lap was completed.
    LapCompleted {
        /// Stable entrant identifier.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// The completed lap record.
        record: LapRecord,
    },
    /// A split was crossed.
    SplitCompleted {
        /// Stable entrant identifier.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// Split index (1–3).
        split: u8,
        /// Split time.
        time: Duration,
    },
    /// A pit stop was completed (`Pit` and `Psf` both received).
    PitStopComplete {
        /// Stable entrant identifier.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// The completed pit stop record.
        record: PitRecord,
    },
    /// A penalty state changed.
    PenaltyChanged {
        /// Stable entrant identifier.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// Previous penalty.
        oldpen: PenaltyInfo,
        /// New penalty.
        newpen: PenaltyInfo,
        /// Reason for the change.
        reason: PenaltyReason,
    },
    /// A driver swap occurred.
    DriverSwap {
        /// Stable entrant identifier.
        id: EntrantId,
        /// LFS player ID (unchanged across swaps).
        plid: PlayerId,
        /// The incoming driver's connection ID.
        new_ucid: ConnectionId,
    },
    /// A player crossed the finish line (provisional).
    Finished {
        /// Stable entrant identifier.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// Total race time.
        ttime: Duration,
        /// Best lap time.
        btime: Duration,
        /// Confirmation flags.
        confirm: RaceConfirmFlags,
    },
    /// A confirmed result arrived.
    ResultConfirmed {
        /// Stable entrant identifier.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// Classified position (0-indexed).
        result_num: u8,
        /// Total number of classified results.
        num_results: u8,
    },
    /// A player left the track without finishing while a race was active.
    Dnf {
        /// Stable entrant identifier.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
    },
    /// A disconnected player rejoined and was matched to their prior
    /// [`super::EntrantState`] by LFS.net username.
    ///
    /// The existing [`EntrantId`] is reused and all prior lap history is
    /// preserved. A [`lap_offset`](super::EntrantState::lap_offset) is applied
    /// so subsequent [`RaceEvent::LapCompleted`] records carry the true
    /// running lap total rather than LFS's reset-to-1 counter.
    ///
    /// Only emitted when a world was created with [`crate::world::World::with_rejoin`].
    EntrantRejoined {
        /// Stable entrant identifier (same as before the disconnect).
        id: EntrantId,
        /// New LFS player ID assigned after reconnect.
        plid: PlayerId,
    },
    /// A driver improved their own best lap time.
    ///
    /// Emitted immediately after [`RaceEvent::LapCompleted`] whenever the
    /// completed lap beats that entrant's previous best. Also fires for their
    /// very first completed lap (since any time beats nothing).
    PersonalBest {
        /// Stable entrant identifier.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// Lap number on which the personal best was set.
        lap: u16,
        /// The new personal best time.
        time: Duration,
        /// The previous personal best, if one existed.
        previous: Option<Duration>,
    },
    /// A new session fastest lap was set.
    ///
    /// Emitted immediately after [`RaceEvent::LapCompleted`] whenever the
    /// completed lap beats the previous best time across all entrants. Also
    /// fires for the very first completed lap of the session (since any time
    /// beats nothing).
    FastestLap {
        /// Stable entrant identifier.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
        /// Lap number on which the fastest lap was set (corrected by any
        /// [`lap_offset`](super::EntrantState::lap_offset)).
        lap: u16,
        /// The fastest lap time.
        time: Duration,
    },
    /// A player teleported to the pits (Shift+P / `Plp` packet).
    ///
    /// The in-progress lap is discarded but the running lap total is left
    /// untouched - LFS does not reset the lap counter on a telepit.
    TeleportedToPits {
        /// Stable entrant identifier.
        id: EntrantId,
        /// LFS player ID.
        plid: PlayerId,
    },
}
