use crate::identifiers::{PlayerId, RequestId};

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Clone,
    Copy,
    Default,
    insim_core::Decode,
    insim_core::Encode,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[non_exhaustive]
/// Penalty state.
pub enum PenaltyInfo {
    /// None, or cleared
    #[default]
    None = 0,

    /// Drive through
    Dt = 1,

    /// Drive Through completed
    DtValid = 2,

    /// Stop go
    Sg = 3,

    /// Stop go completed
    SgValid = 4,

    /// 30 Seconds added
    Seconds30 = 5,

    /// 45 seconds added
    Seconds45 = 6,
}

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
/// Reason for a penalty change.
pub enum PenaltyReason {
    /// Unknown or cleared penalty
    #[default]
    Unknown = 0,

    /// Penalty given by admin
    Admin = 1,

    /// Driving wrong way
    WrongWay = 2,

    /// False start
    FalseStart = 3,

    /// Speeding in pit lane
    Speeding = 4,

    /// Stop-go in pit stop too short
    StopShort = 5,

    /// Compulsory stop is too late
    StopLate = 6,
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Penalty state change for a player.
///
/// - Reports a penalty being applied, updated, or cleared.
pub struct Pen {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player whose penalty changed.
    pub plid: PlayerId,

    /// Previous penalty state.
    pub oldpen: PenaltyInfo,

    /// New penalty state.
    pub newpen: PenaltyInfo,

    /// Reason for the change.
    #[insim(pad_after = 1)]
    pub reason: PenaltyReason,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pen() {
        assert_from_to_bytes!(
            Pen,
            [
                0, // reqi
                3, // plid
                1, // oldpen
                2, // newpen
                4, // reason
                0, // sp3
            ],
            |pen: Pen| {
                assert_eq!(pen.plid, PlayerId(3));
                assert_eq!(pen.oldpen, PenaltyInfo::Dt);
                assert_eq!(pen.newpen, PenaltyInfo::DtValid);
                assert!(matches!(pen.reason, PenaltyReason::Speeding));
            }
        );
    }
}
