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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let plid = PlayerId::decode(buf).map_err(|e| e.nested().context("ConInfo::plid"))?;
        let info = CompCarInfo::decode(buf).map_err(|e| e.nested().context("ConInfo::info"))?;
        // pad 1 bytes
        buf.advance(1);
        let steer = u8::decode(buf).map_err(|e| e.nested().context("ConInfo::steer"))?;

        let thrbrk = u8::decode(buf).map_err(|e| e.nested().context("ConInfo::thrbrk"))?;
        let thr: u8 = (thrbrk >> 4) & 0x0F; // upper 4 bits
        let brk: u8 = thrbrk & 0x0F; // lower 4 bits

        let cluhan = u8::decode(buf).map_err(|e| e.nested().context("ConInfo::cluhan"))?;
        let clu: u8 = (cluhan >> 4) & 0x0F; // upper 4 bits
        let han: u8 = cluhan & 0x0F; // lower 4 bits

        let gearsp = u8::decode(buf).map_err(|e| e.nested().context("ConInfo::gearsp"))?;
        let gearsp = (gearsp >> 4) & 0x0F; // gearsp is only first 4 bits

        let speed = Speed::from_meters_per_sec(
            u8::decode(buf).map_err(|e| e.nested().context("ConInfo::speed"))? as f32,
        );

        let direction_raw =
            u8::decode(buf).map_err(|e| e.nested().context("ConInfo::direction_raw"))?;
        let direction = Heading::from_degrees((direction_raw as f64) * CONINFO_DEGREES_PER_UNIT);

        let heading_raw =
            u8::decode(buf).map_err(|e| e.nested().context("ConInfo::heading_raw"))?;
        let heading = Heading::from_degrees((heading_raw as f64) * CONINFO_DEGREES_PER_UNIT);

        let accelf = u8::decode(buf).map_err(|e| e.nested().context("ConInfo::accelf"))?;
        let accelr = u8::decode(buf).map_err(|e| e.nested().context("ConInfo::accelr"))?;

        let x = i16::decode(buf).map_err(|e| e.nested().context("ConInfo::x"))?;
        let y = i16::decode(buf).map_err(|e| e.nested().context("ConInfo::y"))?;

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
        self.plid
            .encode(buf)
            .map_err(|e| e.nested().context("ConInfo::plid"))?;
        self.info
            .encode(buf)
            .map_err(|e| e.nested().context("ConInfo::info"))?;
        0_u8.encode(buf)
            .map_err(|e| e.nested().context("ConInfo::pad"))?; // pad 1 bytes
        self.steer
            .encode(buf)
            .map_err(|e| e.nested().context("ConInfo::steer"))?;

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
        thrbrk
            .encode(buf)
            .map_err(|e| e.nested().context("ConInfo::thrbrk"))?;

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
        cluhan
            .encode(buf)
            .map_err(|e| e.nested().context("ConInfo::cluhan"))?;

        if self.gearsp > 15 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 15,
                found: self.gearsp as usize,
            }
            .context("ConInfo::gearsp"));
        }

        let gearsp = self.gearsp << 4;
        gearsp
            .encode(buf)
            .map_err(|e| e.nested().context("ConInfo::gearsp"))?;

        (self.speed.to_meters_per_sec() as u8)
            .encode(buf)
            .map_err(|e| e.nested().context("ConInfo::speed"))?;

        let direction_units = (self.direction.to_degrees() / CONINFO_DEGREES_PER_UNIT)
            .round()
            .clamp(0.0, 255.0) as u8;
        direction_units
            .encode(buf)
            .map_err(|e| e.nested().context("ConInfo::direction"))?;

        let heading_units = (self.heading.to_degrees() / CONINFO_DEGREES_PER_UNIT)
            .round()
            .clamp(0.0, 255.0) as u8;
        heading_units
            .encode(buf)
            .map_err(|e| e.nested().context("ConInfo::heading"))?;

        self.accelf
            .encode(buf)
            .map_err(|e| e.nested().context("ConInfo::accelf"))?;
        self.accelr
            .encode(buf)
            .map_err(|e| e.nested().context("ConInfo::accelr"))?;
        self.x
            .encode(buf)
            .map_err(|e| e.nested().context("ConInfo::x"))?;
        self.y
            .encode(buf)
            .map_err(|e| e.nested().context("ConInfo::y"))?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf).map_err(|e| e.nested().context("Con::reqi"))?;
        buf.advance(1);
        let spclose = spclose_strip_high_bits(
            u16::decode(buf).map_err(|e| e.nested().context("Con::spclose"))?,
        );
        let spclose = Speed::from_meters_per_sec(spclose as f32 / 10.0);
        buf.advance(2);
        let time = u32::decode(buf).map_err(|e| e.nested().context("Con::time"))? as u64;
        let time = Duration::from_millis(time);

        let a = ConInfo::decode(buf).map_err(|e| e.nested().context("Con::a"))?;
        let b = ConInfo::decode(buf).map_err(|e| e.nested().context("Con::b"))?;

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
        self.reqi
            .encode(buf)
            .map_err(|e| e.nested().context("Con::reqi"))?;
        buf.put_bytes(0, 1);
        spclose_strip_high_bits((self.spclose.to_meters_per_sec() * 10.0) as u16)
            .encode(buf)
            .map_err(|e| e.nested().context("Con::spclose"))?;
        match TryInto::<u32>::try_into(self.time.as_millis()) {
            Ok(time) => time
                .encode(buf)
                .map_err(|e| e.nested().context("Con::time"))?,
            Err(_) => {
                return Err(insim_core::EncodeErrorKind::OutOfRange {
                    min: 0,
                    max: u32::MAX as usize,
                    found: self.time.as_millis() as usize,
                }
                .context("Con::spclose"));
            },
        }
        self.a
            .encode(buf)
            .map_err(|e| e.nested().context("Con::a"))?;
        self.b
            .encode(buf)
            .map_err(|e| e.nested().context("Con::b"))?;
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
