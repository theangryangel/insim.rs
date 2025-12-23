use std::time::Duration;

use bytes::{Buf, BufMut};
use insim_core::{direction::Heading, coordinate::Coordinate, Decode, Encode};

use super::{CameraView, StaFlags};
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Camera Position Pack reports the current camera position and state. This packet may also be
/// sent to control the camera.
pub struct Cpp {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Position vector
    pub pos: Coordinate,

    /// heading - 0 points along Y axis
    pub h: Heading,

    /// Pitch
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
    pub time: Duration,

    /// State flags to set
    pub flags: StaFlags,
}

impl Decode for Cpp {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf)?;
        buf.advance(1);
        let pos = Coordinate::decode(buf)?;

        let h = Heading::from_degrees((u16::decode(buf)? as f64) * super::mci::COMPCAR_DEGREES_PER_UNIT);
        let p = u16::decode(buf)?;
        let r = u16::decode(buf)?;

        let viewplid = PlayerId::decode(buf)?;
        let ingamecam = CameraView::decode(buf)?;

        let fov = f32::decode(buf)?;
        let time = Duration::from_millis(u16::decode(buf)? as u64);
        let flags = StaFlags::decode(buf)?;

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
            flags
        })

    }
}

impl Encode for Cpp {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi.encode(buf)?;
        buf.put_bytes(0, 1);
        self.pos.encode(buf)?;
        let h = (self.h.to_degrees() / super::mci::COMPCAR_DEGREES_PER_UNIT)
            .round()
            .clamp(0.0, 65535.0) as u16;
        h.encode(buf)?;
        self.p.encode(buf)?;
        self.r.encode(buf)?;
        self.viewplid.encode(buf)?;
        self.ingamecam.encode(buf)?;
        self.fov.encode(buf)?;
        (self.time.as_millis() as u16).encode(buf)?;
        self.flags.encode(buf)?;

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
                1,   // reqi
                0,   // zero
                1,   // x (1)
                0,   // x (2)
                0,   // x (3)
                0,   // x (4)
                255, // y (1)
                255, // y (2)
                255, // y (3)
                127, // y (4)
                0,   // z (1)
                0,   // z (2)
                0,   // z (3)
                128, // z (4)
                255, // h (1)
                255, // h (2)
                200, // p (1)
                1,   // p (2)
                39,  // r (1)
                0,   // r (0)
                32,  // viewplid
                4,   // ingamecam
                0,   // fov (1)
                0,   // fov (2)
                32,  // fov (3)
                66,  // fov (4)
                200, // time (1)
                0,   // time (2)
                0,   // flags (1)
                32,  // flags (2)
            ],
            |parsed: Cpp| {
                assert_eq!(parsed.reqi, RequestId(1));
                assert_eq!(parsed.time, Duration::from_millis(200));
            }
        )
    }
}
