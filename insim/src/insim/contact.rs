// con is a reserved word. Do not name this file `con.rs`.
use std::time::Duration;

use bytes::{Buf, BufMut};
use insim_core::{
    direction::{Direction, DirectionKind},
    speed::{Speed, SpeedKind},
    Decode, Encode,
};

use super::{obh::spclose_strip_high_bits, CompCarInfo};
use crate::identifiers::{PlayerId, RequestId};

#[derive(Copy, Clone, Debug, Default)]
pub struct SpeedConInfo;

impl SpeedKind for SpeedConInfo {
    type Inner = u8;

    fn name() -> &'static str {
        "m/s"
    }

    fn from_meters_per_sec(value: f32) -> Self::Inner {
        value as Self::Inner
    }

    fn to_meters_per_sec(value: Self::Inner) -> f32 {
        value as f32
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct DirectionConInfo;

impl DirectionKind for DirectionConInfo {
    type Inner = u8;

    fn name() -> &'static str {
        "128 = 180 deg"
    }

    fn from_radians(value: f32) -> Self::Inner {
        ((value * 128.0 / std::f32::consts::PI)
            .round()
            .clamp(0.0, 255.0)) as u8
    }

    fn to_radians(value: Self::Inner) -> f32 {
        (value as f32) * std::f32::consts::PI / 128.0
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Used within [Con] packet to give a break down of information about the Contact between the two
/// players.
pub struct ConInfo {
    /// Unique player id
    pub plid: PlayerId,

    /// Additional information
    pub info: CompCarInfo,

    /// Front wheel steer in degrees (right positive)
    pub steer: u8,

    /// Throttle - Insim defines this as a u4, insim.rs will silently truncate this u8.
    pub thr: u8,

    /// Brake - Insim defines this as a u4, insim.rs will validate this on encoding.
    pub brk: u8,

    /// Clutch (0-15) - Insim defines this as a u4, insim.rs will validate this on encoding.
    pub clu: u8,

    /// Handbrake - Insim defines this as a u4, insim.rs will validate this on encoding.
    pub han: u8,

    /// Gear (15=R) - Insim defines this as a u4, insim.rs will validate this on encoding.
    pub gearsp: u8,

    /// Speed in m/s
    pub speed: Speed<SpeedConInfo>,

    /// Car's motion if Speed > 0: 0 = world y direction, 128 = 180 deg
    pub direction: Direction<DirectionConInfo>,

    /// direction of forward axis: 0 = world y direction, 128 = 180 deg
    pub heading: Direction<DirectionConInfo>,

    /// m/s^2 longitudinal acceleration (forward positive)
    pub accelf: u8,

    /// m/s^2 lateral acceleration (right positive)
    pub accelr: u8,

    /// position (1 metre = 16)
    pub x: i16,

    /// position (1 metre = 16)
    pub y: i16,
}

impl Decode for ConInfo {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let plid = PlayerId::decode(buf)?;
        let info = CompCarInfo::decode(buf)?;
        // pad 1 bytes
        buf.advance(1);
        let steer = u8::decode(buf)?;

        let thrbrk = u8::decode(buf)?;
        let thr: u8 = (thrbrk >> 4) & 0x0F; // upper 4 bits
        let brk: u8 = thrbrk & 0x0F; // lower 4 bits

        let cluhan = u8::decode(buf)?;
        let clu: u8 = (cluhan >> 4) & 0x0F; // upper 4 bits
        let han: u8 = cluhan & 0x0F; // lower 4 bits

        let gearsp = u8::decode(buf)?;
        let gearsp = (gearsp >> 4) & 0x0F; // gearsp is only first 4 bits

        let speed = Speed::decode(buf)?;
        let direction = Direction::decode(buf)?;
        let heading = Direction::decode(buf)?;
        let accelf = u8::decode(buf)?;
        let accelr = u8::decode(buf)?;

        let x = i16::decode(buf)?;
        let y = i16::decode(buf)?;

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
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.plid.encode(buf)?;
        self.info.encode(buf)?;
        0_u8.encode(buf)?; // pad 1 bytes
        self.steer.encode(buf)?;

        if self.thr > 15 {
            return Err(insim_core::EncodeError::TooLarge);
        }

        if self.brk > 15 {
            return Err(insim_core::EncodeError::TooLarge);
        }

        let thrbrk = (self.thr << 4) | self.brk;
        thrbrk.encode(buf)?;

        if self.clu > 15 {
            return Err(insim_core::EncodeError::TooLarge);
        }

        if self.han > 15 {
            return Err(insim_core::EncodeError::TooLarge);
        }

        let cluhan = (self.clu << 4) | self.han;
        cluhan.encode(buf)?;

        if self.gearsp > 15 {
            return Err(insim_core::EncodeError::TooLarge);
        }

        let gearsp = self.gearsp << 4;
        gearsp.encode(buf)?;

        self.speed.encode(buf)?;
        self.direction.encode(buf)?;
        self.heading.encode(buf)?;
        self.accelf.encode(buf)?;
        self.accelr.encode(buf)?;
        self.x.encode(buf)?;
        self.y.encode(buf)?;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ClosingSpeed;

impl SpeedKind for ClosingSpeed {
    type Inner = u16;

    fn name() -> &'static str {
        "closing speed (10 = 1 m/s)"
    }

    fn from_meters_per_sec(value: f32) -> Self::Inner {
        (value / 10.0) as Self::Inner
    }

    fn to_meters_per_sec(value: Self::Inner) -> f32 {
        (value * 10) as f32
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Contact between 2 vehicles
pub struct Con {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Low 12 bits: closing speed (10 = 1 m/s)
    /// The high 4 bits are automatically stripped.
    pub spclose: Speed<ClosingSpeed>,

    /// Time since last reset. Warning this is looping.
    pub time: Duration,

    /// Contact information for vehicle A
    pub a: ConInfo,

    /// Contact information for vehicle B
    pub b: ConInfo,
}

impl Decode for Con {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf)?;
        buf.advance(1);
        let spclose = spclose_strip_high_bits(u16::decode(buf)?);
        let spclose = Speed::new(spclose);
        let time = u16::decode(buf)? as u64;
        let time = Duration::from_millis(time * 10);

        let a = ConInfo::decode(buf)?;
        let b = ConInfo::decode(buf)?;

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
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi.encode(buf)?;
        buf.put_bytes(0, 1);
        spclose_strip_high_bits(self.spclose.into_inner()).encode(buf)?;
        match TryInto::<u16>::try_into(self.time.as_millis() / 10) {
            Ok(time) => time.encode(buf)?,
            Err(_) => return Err(insim_core::EncodeError::TooLarge),
        }
        self.a.encode(buf)?;
        self.b.encode(buf)?;
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
