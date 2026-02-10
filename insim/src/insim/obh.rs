use std::time::Duration;

use bitflags::bitflags;
use bytes::{Buf, BufMut};
use insim_core::{Decode, Encode, heading::Heading, speed::Speed};

use crate::identifiers::{PlayerId, RequestId};

/// CarContact direction scale: 128 units = 180°
const CARCONTACT_DEGREES_PER_UNIT: f64 = 180.0 / 128.0;

pub(crate) fn spclose_strip_high_bits(val: u16) -> u16 {
    val & !61440
}

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    /// Additional information for an object hit.
    pub struct ObhFlags: u8 {
        /// An added object was hit
        const LAYOUT = (1 << 0);
        /// A movable object was hit
        const CAN_MOVE = (1 << 1);
        /// The object was in motion
        const WAS_MOVING = (1 << 2);
        /// The object was in it's original position
        const ON_SPOT = (1 << 3);
    }
}

generate_bitflag_helpers! {
    ObhFlags,

    pub is_layout_object => LAYOUT,
    pub is_movable_object => CAN_MOVE,
    pub was_moving => WAS_MOVING,
    pub was_in_original_position => ON_SPOT
}

impl_bitflags_from_to_bytes!(ObhFlags, u8);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Contact details used by collision reports.
pub struct CarContact {
    /// Direction of motion.
    pub direction: Heading,

    /// Car facing direction.
    pub heading: Heading,

    /// Speed.
    pub speed: Speed,

    /// Z position.
    pub z: u8,

    /// X position.
    pub x: i16,

    /// Y position.
    pub y: i16,
}

impl Decode for CarContact {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let direction_raw =
            u8::decode(buf).map_err(|e| e.nested().context("CarContact::direction_raw"))?;
        let direction = Heading::from_degrees((direction_raw as f64) * CARCONTACT_DEGREES_PER_UNIT);

        let heading_raw =
            u8::decode(buf).map_err(|e| e.nested().context("CarContact::heading_raw"))?;
        let heading = Heading::from_degrees((heading_raw as f64) * CARCONTACT_DEGREES_PER_UNIT);

        let speed = u8::decode(buf).map_err(|e| e.nested().context("CarContact::speed"))?;
        let speed = Speed::from_meters_per_sec(speed as f32);
        let z = u8::decode(buf).map_err(|e| e.nested().context("CarContact::z"))?;
        let x = i16::decode(buf).map_err(|e| e.nested().context("CarContact::x"))?;
        let y = i16::decode(buf).map_err(|e| e.nested().context("CarContact::y"))?;
        Ok(Self {
            direction,
            heading,
            speed,
            z,
            x,
            y,
        })
    }
}

impl Encode for CarContact {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        let direction_units = (self.direction.to_degrees() / CARCONTACT_DEGREES_PER_UNIT)
            .round()
            .clamp(0.0, 255.0) as u8;
        direction_units
            .encode(buf)
            .map_err(|e| e.nested().context("CarContact::direction"))?;

        let heading_units = (self.heading.to_degrees() / CARCONTACT_DEGREES_PER_UNIT)
            .round()
            .clamp(0.0, 255.0) as u8;
        heading_units
            .encode(buf)
            .map_err(|e| e.nested().context("CarContact::heading"))?;

        (self.speed.to_meters_per_sec() as u8)
            .encode(buf)
            .map_err(|e| e.nested().context("CarContact::speed"))?;
        self.z
            .encode(buf)
            .map_err(|e| e.nested().context("CarContact::z"))?;
        self.x
            .encode(buf)
            .map_err(|e| e.nested().context("CarContact::x"))?;
        self.y
            .encode(buf)
            .map_err(|e| e.nested().context("CarContact::y"))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Object hit report.
///
/// - Sent when object hit reporting is enabled in [IsiFlags](crate::insim::IsiFlags).
pub struct Obh {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player that hit the object.
    pub plid: PlayerId,

    /// Closing speed at impact.
    pub spclose: Speed,

    /// Time since session start (wraps periodically).
    pub time: Duration,

    /// Contact details.
    pub c: CarContact,

    /// Object X position.
    pub x: i16,

    /// Object Y position.
    pub y: i16,

    /// Object Z position.
    pub zbyte: u8,

    /// Object index in the layout.
    pub index: u8,

    /// Additional object flags.
    pub flags: ObhFlags,
}

impl Decode for Obh {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf).map_err(|e| e.nested().context("Obh::reqi"))?;
        let plid = PlayerId::decode(buf).map_err(|e| e.nested().context("Obh::plid"))?;
        // automatically strip off the first 4 bits as they're reserved
        let spclose = spclose_strip_high_bits(
            u16::decode(buf).map_err(|e| e.nested().context("Obh::spclose"))?,
        );
        let spclose = Speed::from_meters_per_sec(spclose as f32 / 10.0);
        buf.advance(2);

        let time = Duration::from_millis(
            u32::decode(buf).map_err(|e| e.nested().context("Obh::time"))? as u64,
        );
        let c = CarContact::decode(buf).map_err(|e| e.nested().context("Obh::c"))?;
        // FIXME: become glam Vec
        let x = i16::decode(buf).map_err(|e| e.nested().context("Obh::x"))?;
        let y = i16::decode(buf).map_err(|e| e.nested().context("Obh::y"))?;
        let zbyte = u8::decode(buf).map_err(|e| e.nested().context("Obh::zbyte"))?;
        buf.advance(1);
        let index = u8::decode(buf).map_err(|e| e.nested().context("Obh::index"))?;
        let flags = ObhFlags::decode(buf).map_err(|e| e.nested().context("Obh::flags"))?;
        Ok(Self {
            reqi,
            plid,
            spclose,
            time,
            c,
            x,
            y,
            zbyte,
            index,
            flags,
        })
    }
}

impl Encode for Obh {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi
            .encode(buf)
            .map_err(|e| e.nested().context("Obh::reqi"))?;
        self.plid
            .encode(buf)
            .map_err(|e| e.nested().context("Obh::plid"))?;
        // automatically strip off the first 4 bits as they're reserved
        let spclose = spclose_strip_high_bits((self.spclose.into_inner() * 10.0) as u16);
        spclose
            .encode(buf)
            .map_err(|e| e.nested().context("Obh::spclose"))?;
        buf.put_bytes(0, 2);
        match u32::try_from(self.time.as_millis()) {
            Ok(time) => time
                .encode(buf)
                .map_err(|e| e.nested().context("Obh::time"))?,
            Err(_) => {
                return Err(insim_core::EncodeErrorKind::OutOfRange {
                    min: 0,
                    max: u32::MAX as usize,
                    found: self.time.as_millis() as usize,
                }
                .context("Obh time out of range"));
            },
        }
        self.c
            .encode(buf)
            .map_err(|e| e.nested().context("Obh::c"))?;
        self.x
            .encode(buf)
            .map_err(|e| e.nested().context("Obh::x"))?;
        self.y
            .encode(buf)
            .map_err(|e| e.nested().context("Obh::y"))?;
        self.zbyte
            .encode(buf)
            .map_err(|e| e.nested().context("Obh::zbyte"))?;
        buf.put_bytes(0, 1);
        self.index
            .encode(buf)
            .map_err(|e| e.nested().context("Obh::index"))?;
        self.flags
            .encode(buf)
            .map_err(|e| e.nested().context("Obh::flags"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obh() {
        assert_from_to_bytes!(
            Obh,
            [
                0,   // reqi
                3,   // plid
                23,  // spclose (1)
                0,   // spclose (2)
                0,   // spw
                0,   // spw
                106, // time (1)
                19,  // time (2)
                0,   // time (3)
                0,   // time (4)
                2,   // c - direction
                254, // c - heading
                3,   // c - speed
                9,   // c - zbyte
                4,   // c - x (1)
                213, // c - x (2)
                132, // c - y (1)
                134, // c - y (2)
                18,  // x (1)
                213, // x (2)
                174, // y (1)
                134, // y (2)
                1,   // zbyte
                0,   // sp1
                113, // index
                11,  // obhflags
            ],
            |obh: Obh| {
                assert_eq!(obh.reqi, RequestId(0));
                assert_eq!(obh.plid, PlayerId(3));
                assert_eq!(obh.time, Duration::from_millis(4970));
                assert_eq!(obh.spclose.into_inner(), 23.0 / 10.0);
            }
        );
    }

    #[test]
    fn ensure_high_bits_stripped() {
        assert_eq!(spclose_strip_high_bits(61441), 1);

        assert_eq!(spclose_strip_high_bits(63495,), 2055);
    }
}
