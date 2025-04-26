use std::time::Duration;

use insim_core::vehicle::Vehicle;

use super::{PlayerFlags, RaceConfirmFlags};
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Race Result - qualifying or confirmed result
pub struct Res {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// The unique player ID that this race result is for
    pub plid: PlayerId,

    #[read_write_buf(codepage(length = 24))]
    /// The LFS.net username of the player
    pub uname: String,

    #[read_write_buf(codepage(length = 24))]
    /// The name of the player
    pub pname: String,

    #[read_write_buf(codepage(length = 8))]
    /// The number plate of the player
    pub plate: String,

    /// The vehicle they finished in
    pub cname: Vehicle,

    #[read_write_buf(duration(milliseconds = u32))]
    /// The total time
    pub ttime: Duration,

    #[read_write_buf(duration(milliseconds = u32), pad_after = 1)]
    /// The best lap time
    pub btime: Duration,

    /// The number of pit stops taken
    pub numstops: u8,

    /// The result flags. Where they DNF?
    #[read_write_buf(pad_after = 1)]
    pub confirm: RaceConfirmFlags,

    /// The number of laps done
    pub lapsdone: u16,

    /// Additional information about the player.
    pub flags: PlayerFlags,

    /// Finish or qualify pos (0 = win / 255 = not added to table)
    pub resultnum: u8,

    /// Total number of results (qualify doesn't always add a new one)
    pub numres: u8,

    /// Penalty time in seconds (already included in race time)
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
