use std::time::Duration;

use bitflags::bitflags;
use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw, BinRead, BinResult},
    duration::{binrw_parse_duration, binrw_write_duration},
    ReadWriteBuf,
};

use crate::identifiers::{PlayerId, RequestId};

fn strip_high_bits(val: u16) -> u16 {
    val & !61440
}

#[binrw::parser(reader, endian)]
pub(crate) fn binrw_parse_spclose_strip_reserved_bits() -> BinResult<u16> {
    let res = u16::read_options(reader, endian, ())?;
    // strip the top 4 bits off
    Ok(strip_high_bits(res))
}

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
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

#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Vehicle made contact with something else
pub struct CarContact {
    /// Car's motion if Speed > 0: 0 = world y direction, 128 = 180 deg
    pub direction: u8,

    /// Direction of forward axis: 0 = world y direction, 128 = 180 deg
    pub heading: u8,

    /// Speed in m/s
    pub speed: u8,

    /// Z position (1 metre = 16)
    pub z: u8,

    /// X position (1 metre = 16)
    pub x: i16,

    /// Y position (1 metre = 16)
    pub y: i16,
}

#[binrw]
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
    #[br(parse_with = binrw_parse_spclose_strip_reserved_bits)]
    pub spclose: u16,

    #[br(parse_with = binrw_parse_duration::<u16, 10, _>)]
    #[bw(write_with = binrw_write_duration::<u16, 10, _>)]
    /// When this occurred. Warning this is looping.
    pub time: Duration,

    /// Additional information about the contact.
    pub c: CarContact,

    /// The X position of the object
    pub x: i16,

    /// The Y position of the object
    pub y: i16,

    #[brw(pad_after = 1)]
    /// The Z position of the object
    pub zbyte: u8,

    /// The object index
    pub index: u8,

    /// Additional flags and information about the object
    pub flags: ObhFlags,
}

impl ReadWriteBuf for Obh {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        let plid = PlayerId::read_buf(buf)?;
        // automatically strip off the first 4 bits as they're reserved
        let spclose = strip_high_bits(u16::read_buf(buf)?);
        let time = Duration::from_millis((u16::read_buf(buf)? as u64) * 10);
        let c = CarContact::read_buf(buf)?;
        let x = i16::read_buf(buf)?;
        let y = i16::read_buf(buf)?;
        let zbyte = u8::read_buf(buf)?;
        buf.advance(1);
        let index = u8::read_buf(buf)?;
        let flags = ObhFlags::read_buf(buf)?;
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

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        self.plid.write_buf(buf)?;
        // automatically strip off the first 4 bits as they're reserved
        strip_high_bits(self.spclose).write_buf(buf)?;
        // FIXME: handle if this is too small
        let time = (self.time.as_millis() / 10) as u16;
        time.write_buf(buf)?;
        self.c.write_buf(buf)?;
        self.x.write_buf(buf)?;
        self.y.write_buf(buf)?;
        self.zbyte.write_buf(buf)?;
        buf.put_bytes(0, 1);
        self.index.write_buf(buf)?;
        self.flags.write_buf(buf)?;
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
                241, // time (1)
                1,   // time (2)
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
                assert_eq!(obh.spclose, 23);
            }
        );
    }

    #[test]
    fn ensure_high_bits_stripped() {
        assert_eq!(strip_high_bits(61441), 1);

        assert_eq!(strip_high_bits(63495,), 2055);
    }
}
