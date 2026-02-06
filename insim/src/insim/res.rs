use std::time::Duration;

use insim_core::vehicle::Vehicle;

use super::{PlayerFlags, RaceConfirmFlags};
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Confirmed race or qualifying result.
///
/// - Includes player identity, car, and timing details.
pub struct Res {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player this result belongs to.
    pub plid: PlayerId,

    #[insim(codepage(length = 24))]
    /// LFS.net username.
    pub uname: String,

    #[insim(codepage(length = 24))]
    /// Player nickname.
    pub pname: String,

    #[insim(codepage(length = 8))]
    /// Number plate.
    pub plate: String,

    /// Vehicle used for the result.
    pub cname: Vehicle,

    #[insim(duration = u32)]
    /// Total time.
    pub ttime: Duration,

    #[insim(duration = u32, pad_after = 1)]
    /// Best lap time.
    pub btime: Duration,

    /// Number of pit stops.
    pub numstops: u8,

    /// Confirmation flags and penalties.
    #[insim(pad_after = 1)]
    pub confirm: RaceConfirmFlags,

    /// Laps completed.
    pub lapsdone: u16,

    /// Player flags (help settings).
    pub flags: PlayerFlags,

    /// Finish or qualify position (0 = win, 255 = not in table).
    pub resultnum: u8,

    /// Total number of results.
    pub numres: u8,

    /// Penalty time (already included in `ttime`).
    pub pseconds: u16,
}

#[cfg(test)]
mod test {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_res() {
        let mut data = BytesMut::new();
        data.extend_from_slice(&[
            0, // reqi
            3, // plud
        ]);
        data.extend_from_slice(b"abc1"); // uname
        data.put_bytes(0, 20);
        data.extend_from_slice(b"def2"); // pname
        data.put_bytes(0, 20);
        data.extend_from_slice(b"12345678"); // plate
        data.extend_from_slice(b"XRT\0"); // skin
        data.extend_from_slice(&[
            234, // TTime (1)
            8,   // TTime (2)
            0,   // TTime (3)
            4,   // TTime (4)
            128, // BTime (1)
            2,   // BTime (2)
            1,   // BTime (3)
            1,   // BTime (4)
            0,   // SpA
            2,   // NumStops
            5,   // Confirm
            0,   // SpB
            68,  // LapsDone (1)
            0,   // LapsDone (2)
            9,   // Flags (1)
            0,   // Flags (2)
            12,  // ResultNum
            22,  // NumRes
            6,   // PSeconds (1)
            0,   // PSeconds (2)
        ]);

        assert_from_to_bytes!(Res, data.as_ref(), |res: Res| {
            assert_eq!(res.cname, Vehicle::Xrt);
        });
    }
}
