// XXX: `con` is a reserved word on Windows.
// Do not name this file `con.rs`.
use std::time::Duration;

use bytes::{Buf, BufMut};
use insim_core::{Decode, Encode, heading::Heading, speed::Speed};

use super::{CompCarInfo, obh::spclose_strip_high_bits};
use crate::identifiers::{PlayerId, RequestId};

/// ConInfo direction scale: 128 units = 180°
const CONINFO_DEGREES_PER_UNIT: f64 = 180.0 / 128.0;

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
    pub speed: Speed,

    /// Car's motion if Speed > 0: 0 = world y direction, 128 = 180 deg
    /// Stored internally as radians
    pub direction: Heading,

    /// direction of forward axis: 0 = world y direction, 128 = 180 deg
    /// Stored internally as radians
    pub heading: Heading,

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

        let speed = Speed::from_meters_per_sec(u8::decode(buf)? as f32);

        let direction_raw = u8::decode(buf)?;
        let direction = Heading::from_degrees((direction_raw as f64) * CONINFO_DEGREES_PER_UNIT);

        let heading_raw = u8::decode(buf)?;
        let heading = Heading::from_degrees((heading_raw as f64) * CONINFO_DEGREES_PER_UNIT);

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
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 15,
                found: self.thr as usize,
            }
            .context("Thr out of range"));
        }

        if self.brk > 15 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 15,
                found: self.brk as usize,
            }
            .context("Brk out of range"));
        }

        let thrbrk = (self.thr << 4) | self.brk;
        thrbrk.encode(buf)?;

        if self.clu > 15 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 15,
                found: self.clu as usize,
            }
            .context("Clu out of range"));
        }

        if self.han > 15 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 15,
                found: self.han as usize,
            }
            .context("Han out of range"));
        }

        let cluhan = (self.clu << 4) | self.han;
        cluhan.encode(buf)?;

        if self.gearsp > 15 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 15,
                found: self.gearsp as usize,
            }
            .context("Gearsp out of range"));
        }

        let gearsp = self.gearsp << 4;
        gearsp.encode(buf)?;

        (self.speed.to_meters_per_sec() as u8).encode(buf)?;

        let direction_units = (self.direction.to_degrees() / CONINFO_DEGREES_PER_UNIT)
            .round()
            .clamp(0.0, 255.0) as u8;
        direction_units.encode(buf)?;

        let heading_units = (self.heading.to_degrees() / CONINFO_DEGREES_PER_UNIT)
            .round()
            .clamp(0.0, 255.0) as u8;
        heading_units.encode(buf)?;

        self.accelf.encode(buf)?;
        self.accelr.encode(buf)?;
        self.x.encode(buf)?;
        self.y.encode(buf)?;

        Ok(())
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
    pub spclose: Speed,

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
        let spclose = Speed::from_meters_per_sec(spclose as f32 / 10.0);
        buf.advance(2);
        let time = u32::decode(buf)? as u64;
        let time = Duration::from_millis(time);

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
        spclose_strip_high_bits((self.spclose.to_meters_per_sec() * 10.0) as u16).encode(buf)?;
        match TryInto::<u32>::try_into(self.time.as_millis()) {
            Ok(time) => time.encode(buf)?,
            Err(_) => {
                return Err(insim_core::EncodeErrorKind::OutOfRange {
                    min: 0,
                    max: u32::MAX as usize,
                    found: self.time.as_millis() as usize,
                }
                .context("Time out of range"));
            },
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
