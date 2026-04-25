// XXX: `con` is a reserved word on Windows.
// Do not name this file `con.rs`.
use std::time::Duration;

use insim_core::{Decode, DecodeContext, Encode, EncodeContext, heading::Heading, speed::Speed};

use super::{CompCarInfo, obh::spclose_strip_high_bits};
use crate::identifiers::{PlayerId, RequestId};

/// ConInfo direction scale: 128 units = 180°
const CONINFO_DEGREES_PER_UNIT: f64 = 180.0 / 128.0;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
/// Per-car contact details used in [Con].
pub struct ConInfo {
    /// Player identifier.
    pub plid: PlayerId,

    /// Additional car state flags.
    pub info: CompCarInfo,

    /// Front wheel steering angle (right positive).
    pub steer: u8,

    /// Throttle input (0-15).
    pub thr: u8,

    /// Brake input (0-15).
    pub brk: u8,

    /// Clutch input (0-15).
    pub clu: u8,

    /// Handbrake input (0-15).
    pub han: u8,

    /// Gear selector (0-15).
    pub gearsp: u8,

    /// Speed.
    pub speed: Speed,

    /// Direction of motion.
    pub direction: Heading,

    /// Car facing direction.
    pub heading: Heading,

    /// Longitudinal acceleration.
    pub accelf: u8,

    /// Lateral acceleration.
    pub accelr: u8,

    /// X position.
    pub x: i16,

    /// Y position.
    pub y: i16,
}

impl Decode for ConInfo {
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let plid = ctx.decode::<PlayerId>("plid")?;
        let info = ctx.decode::<CompCarInfo>("info")?;
        // pad 1 bytes
        ctx.pad("sp0", 1)?;
        let steer = ctx.decode::<u8>("steer")?;

        let thrbrk = ctx.decode::<u8>("thrbrk")?;
        let thr: u8 = (thrbrk >> 4) & 0x0F; // upper 4 bits
        let brk: u8 = thrbrk & 0x0F; // lower 4 bits

        let cluhan = ctx.decode::<u8>("cluhan")?;
        let clu: u8 = (cluhan >> 4) & 0x0F; // upper 4 bits
        let han: u8 = cluhan & 0x0F; // lower 4 bits

        let gearsp = ctx.decode::<u8>("gearsp")?;
        let gearsp = (gearsp >> 4) & 0x0F; // gearsp is only first 4 bits

        let speed = Speed::from_meters_per_sec(ctx.decode::<u8>("speed")? as f32);

        let direction_raw = ctx.decode::<u8>("direction_raw")?;
        let direction = Heading::from_degrees((direction_raw as f64) * CONINFO_DEGREES_PER_UNIT);

        let heading_raw = ctx.decode::<u8>("heading_raw")?;
        let heading = Heading::from_degrees((heading_raw as f64) * CONINFO_DEGREES_PER_UNIT);

        let accelf = ctx.decode::<u8>("accelf")?;
        let accelr = ctx.decode::<u8>("accelr")?;

        let x = ctx.decode::<i16>("x")?;
        let y = ctx.decode::<i16>("y")?;

        Ok(Self {
            plid,
            info,
            steer,
            thr,
            brk,
            clu,
            han,
            gearsp,
            speed,
            direction,
            heading,
            accelf,
            accelr,
            x,
            y,
        })
    }
}

impl Encode for ConInfo {
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.encode("plid", &self.plid)?;
        ctx.encode("info", &self.info)?;
        ctx.pad("sp0", 1)?; // pad 1 bytes
        ctx.encode("steer", &self.steer)?;

        if self.thr > 15 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 15,
                found: self.thr as usize,
            }
            .context("ConInfo::thr"));
        }

        if self.brk > 15 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 15,
                found: self.brk as usize,
            }
            .context("ConInfo::brk"));
        }

        let thrbrk = (self.thr << 4) | self.brk;
        ctx.encode("thrbrk", &thrbrk)?;

        if self.clu > 15 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 15,
                found: self.clu as usize,
            }
            .context("ConInfo::clu"));
        }

        if self.han > 15 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 15,
                found: self.han as usize,
            }
            .context("ConInfo::han"));
        }

        let cluhan = (self.clu << 4) | self.han;
        ctx.encode("cluhan", &cluhan)?;

        if self.gearsp > 15 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 15,
                found: self.gearsp as usize,
            }
            .context("ConInfo::gearsp"));
        }

        let gearsp = self.gearsp << 4;
        ctx.encode("gearsp", &gearsp)?;

        ctx.encode("speed", &(self.speed.to_meters_per_sec() as u8))?;

        let direction_units = (self.direction.to_degrees() / CONINFO_DEGREES_PER_UNIT)
            .round()
            .clamp(0.0, 255.0) as u8;
        ctx.encode("direction", &direction_units)?;

        let heading_units = (self.heading.to_degrees() / CONINFO_DEGREES_PER_UNIT)
            .round()
            .clamp(0.0, 255.0) as u8;
        ctx.encode("heading", &heading_units)?;

        ctx.encode("accelf", &self.accelf)?;
        ctx.encode("accelr", &self.accelr)?;
        ctx.encode("x", &self.x)?;
        ctx.encode("y", &self.y)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
/// Vehicle-to-vehicle contact report.
///
/// - Sent when collision reporting is enabled in [IsiFlags](crate::insim::IsiFlags).
pub struct Con {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Closing speed at impact.
    pub spclose: Speed,

    /// Time since session start (wraps periodically).
    pub time: Duration,

    /// Contact information for vehicle A.
    pub a: ConInfo,

    /// Contact information for vehicle B.
    pub b: ConInfo,
}

impl Decode for Con {
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let reqi = ctx.decode::<RequestId>("reqi")?;
        ctx.pad("sp0", 1)?;
        let spclose = spclose_strip_high_bits(ctx.decode::<u16>("spclose")?);
        let spclose = Speed::from_meters_per_sec(spclose as f32 / 10.0);
        ctx.pad("spw", 2)?;
        let time = ctx.decode_duration::<u32>("time")?;

        let a = ctx.decode::<ConInfo>("a")?;
        let b = ctx.decode::<ConInfo>("b")?;

        Ok(Self {
            reqi,
            spclose,
            time,
            a,
            b,
        })
    }
}

impl Encode for Con {
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.encode("reqi", &self.reqi)?;
        ctx.pad("sp0", 1)?;
        ctx.encode(
            "spclose",
            &spclose_strip_high_bits((self.spclose.to_meters_per_sec() * 10.0) as u16),
        )?;
        ctx.encode_duration::<u32>("time", self.time)?;
        ctx.encode("a", &self.a)?;
        ctx.encode("b", &self.b)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coninfo() {
        // ConInfo has some fields which are effectively u4.
        // We need to ensure that we carefully decode them.
        assert_from_to_bytes!(
            ConInfo,
            [
                1, 0,          // CompCarInfoinfo
                0,          // padding
                12,         // steering
                247,        // thrbrk
                188,        // cluhan
                0b11110000, // gearsp
                0,          //speed
                0,          // direction
                1,          // heading
                2,          // accelf
                3,          // accelr
                0, 0, // X
                0, 0, // Y
            ],
            |coninfo: ConInfo| {
                assert_eq!(coninfo.thr, 15);
                assert_eq!(coninfo.brk, 7);

                assert_eq!(coninfo.clu, 11);
                assert_eq!(coninfo.han, 12);

                assert_eq!(coninfo.gearsp, 15);
            }
        );
    }
}
