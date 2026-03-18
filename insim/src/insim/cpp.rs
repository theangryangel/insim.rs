use std::time::Duration;

use insim_core::{Decode, DecodeContext, Encode, EncodeContext, coordinate::Coordinate, heading::Heading};

use super::{CameraView, StaFlags};
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let reqi = ctx.decode::<RequestId>("reqi")?;
        ctx.pad("zero", 1)?;
        let pos = ctx.decode::<Coordinate>("pos")?;

        let h = Heading::from_degrees(
            (ctx.decode::<u16>("h")? as f64) * super::mci::COMPCAR_DEGREES_PER_UNIT,
        );
        let p = ctx.decode::<u16>("p")?;
        let r = ctx.decode::<u16>("r")?;

        let viewplid = ctx.decode::<PlayerId>("viewplid")?;
        let ingamecam = ctx.decode::<CameraView>("ingamecam")?;

        let fov = ctx.decode::<f32>("fov")?;
        let time = ctx.decode_duration::<u16>("time")?;
        let flags = ctx.decode::<StaFlags>("flags")?;

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
}

impl Encode for Cpp {
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.encode("reqi", &self.reqi)?;
        ctx.pad("zero", 1)?;
        ctx.encode("pos", &self.pos)?;
        let h = (self.h.to_degrees() / super::mci::COMPCAR_DEGREES_PER_UNIT)
            .round()
            .clamp(0.0, 65535.0) as u16;
        ctx.encode("h", &h)?;
        ctx.encode("p", &self.p)?;
        ctx.encode("r", &self.r)?;
        ctx.encode("viewplid", &self.viewplid)?;
        ctx.encode("ingamecam", &self.ingamecam)?;
        ctx.encode("fov", &self.fov)?;
        ctx.encode_duration::<u16>("time", self.time)?;
        ctx.encode("flags", &self.flags)?;

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
