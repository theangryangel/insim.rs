use std::time::Duration;

use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
    point::Point,
    FromToBytes,
};

use super::{CameraView, StaFlags};
use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Camera Position Pack reports the current camera position and state. This packet may also be
/// sent to control the camera.
pub struct Cpp {
    #[brw(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Position vector
    pub pos: Point<i32>,

    /// heading - 0 points along Y axis
    pub h: u16,

    /// Patch
    pub p: u16,

    /// Roll
    pub r: u16,

    /// Unique ID of viewed player (0 = none)
    pub viewplid: PlayerId,

    /// CameraView
    pub ingamecam: CameraView,

    /// Field of View, in degrees
    pub fov: f32,

    /// Time in ms to get there (0 means instant)
    #[br(parse_with = binrw_parse_duration::<u16, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u16, 1, _>)]
    pub time: Duration,

    /// State flags to set
    pub flags: StaFlags,
}

impl FromToBytes for Cpp {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::from_bytes(buf)?;
        buf.advance(1);
        let pos = Point::from_bytes(buf)?;
        let h = u16::from_bytes(buf)?;
        let p = u16::from_bytes(buf)?;
        let r = u16::from_bytes(buf)?;
        let viewplid = PlayerId::from_bytes(buf)?;
        let ingamecam = CameraView::from_bytes(buf)?;
        let fov = f32::from_bytes(buf)?;
        let time = u16::from_bytes(buf)?;
        let time = Duration::from_millis(time as u64);
        let flags = StaFlags::from_bytes(buf)?;
        Ok(Self {
            reqi,
            pos,
            h,
            p,
            r,
            viewplid,
            ingamecam,
            fov,
            time,
            flags,
        })
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.to_bytes(buf)?;
        buf.put_bytes(0, 1);
        self.pos.to_bytes(buf)?;
        self.h.to_bytes(buf)?;
        self.p.to_bytes(buf)?;
        self.r.to_bytes(buf)?;
        self.viewplid.to_bytes(buf)?;
        self.ingamecam.to_bytes(buf)?;
        self.fov.to_bytes(buf)?;
        match TryInto::<u16>::try_into(self.time.as_millis()) {
            Ok(time) => {
                time.to_bytes(buf)?;
            },
            Err(_) => {
                return Err(insim_core::Error::DurationTooLarge);
            },
        };
        self.flags.to_bytes(buf)?;
        Ok(())
    }
}

impl_typical_with_request_id!(Cpp);

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_cpp() {
        assert_from_to_bytes!(
            Cpp,
            [
                1,   // ReqI
                0,   // Zero
                1,   // X (1)
                0,   // X (2)
                0,   // X (3)
                0,   // X (4)
                255, // Y (1)
                255, // Y (2)
                255, // Y (3)
                127, // Y (4)
                0,   // Z (1)
                0,   // Z (2)
                0,   // Z (3)
                128, // Z (4)
                255, // H (1)
                255, // H (2)
                200, // P (1)
                1,   // P (2)
                39,  // R (1)
                0,   // R (0)
                32,  // ViewPLID
                4,   // InGameCam
                0,   // FOV (1)
                0,   // FOV (2)
                32,  // FOV (3)
                66,  // FOV (4)
                200, // Time (1)
                0,   // Time (2)
                0,   // Flags (1)
                32,  // Flags (2)
            ],
            |parsed: Cpp| {
                assert_eq!(parsed.reqi, RequestId(1));
                assert_eq!(parsed.time, Duration::from_millis(200));
            }
        )
    }
}
