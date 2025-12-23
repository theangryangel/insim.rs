use std::time::Duration;

use bitflags::bitflags;
use bytes::{Buf, BufMut};
use insim_core::{Decode, Encode, direction::Heading, speed::Speed};

use crate::identifiers::{PlayerId, RequestId};

/// CarContact direction scale: 128 units = 180°
const CARCONTACT_DEGREES_PER_UNIT: f64 = 180.0 / 128.0;

pub(crate) fn spclose_strip_high_bits(val: u16) -> u16 {
    val & !61440
}

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    /// Additional information for the object hit, used within the [Obh] packet.
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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Vehicle made contact with something else
pub struct CarContact {
    /// Car's motion if Speed > 0: 0 = world y direction
    /// Stored internally as radians
    pub direction: Heading,

    /// Direction of forward axis: 0 = world y direction
    /// Stored internally as radians
    pub heading: Heading,

    /// Speed in m/s
    pub speed: Speed,

    /// Z position (1 metre = 16)
    pub z: u8,

    /// X position (1 metre = 16)
    pub x: i16,

    /// Y position (1 metre = 16)
    pub y: i16,
}

impl Decode for CarContact {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let direction_raw = u8::decode(buf)?;
        let direction =
            Heading::from_degrees((direction_raw as f64) * CARCONTACT_DEGREES_PER_UNIT);

        let heading_raw = u8::decode(buf)?;
        let heading = Heading::from_degrees((heading_raw as f64) * CARCONTACT_DEGREES_PER_UNIT);

        let speed = u8::decode(buf)?;
        let speed = Speed::from_meters_per_sec(speed as f32);
        let z = u8::decode(buf)?;
        let x = i16::decode(buf)?;
        let y = i16::decode(buf)?;
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
        direction_units.encode(buf)?;

        let heading_units = (self.heading.to_degrees() / CARCONTACT_DEGREES_PER_UNIT)
            .round()
            .clamp(0.0, 255.0) as u8;
        heading_units.encode(buf)?;

        (self.speed.to_meters_per_sec() as u8).encode(buf)?;
        self.z.encode(buf)?;
        self.x.encode(buf)?;
        self.y.encode(buf)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Object Hit
pub struct Obh {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player id
    pub plid: PlayerId,

    /// Low 12 bits: closing speed (10 = 1 m/s)
    /// The high 4 bits are automatically stripped.
    pub spclose: Speed,

    /// When this occurred. Warning this is looping.
    pub time: Duration,

    /// Additional information about the contact.
    pub c: CarContact,

    /// The X position of the object
    pub x: i16,

    /// The Y position of the object
    pub y: i16,

    /// The Z position of the object
    pub zbyte: u8,

    /// The object index
    pub index: u8,

    /// Additional flags and information about the object
    pub flags: ObhFlags,
}

impl Decode for Obh {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf)?;
        let plid = PlayerId::decode(buf)?;
        // automatically strip off the first 4 bits as they're reserved
        let spclose = spclose_strip_high_bits(u16::decode(buf)?);
        let spclose = Speed::from_meters_per_sec(spclose as f32 / 10.0);
        buf.advance(2);

        let time = Duration::from_millis(u32::decode(buf)? as u64);
        let c = CarContact::decode(buf)?;
        // FIXME: become glam Vec
        let x = i16::decode(buf)?;
        let y = i16::decode(buf)?;
        let zbyte = u8::decode(buf)?;
        buf.advance(1);
        let index = u8::decode(buf)?;
        let flags = ObhFlags::decode(buf)?;
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
        self.reqi.encode(buf)?;
        self.plid.encode(buf)?;
        // automatically strip off the first 4 bits as they're reserved
        let spclose = spclose_strip_high_bits((self.spclose.into_inner() * 10.0) as u16);
        spclose.encode(buf)?;
        buf.put_bytes(0, 2);
        match u32::try_from(self.time.as_millis()) {
            Ok(time) => time.encode(buf)?,
            Err(_) => return Err(insim_core::EncodeError::TooLarge),
        }
        self.c.encode(buf)?;
        self.x.encode(buf)?;
        self.y.encode(buf)?;
        self.zbyte.encode(buf)?;
        buf.put_bytes(0, 1);
        self.index.encode(buf)?;
        self.flags.encode(buf)?;
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
