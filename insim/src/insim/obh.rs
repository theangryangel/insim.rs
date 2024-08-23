use std::time::Duration;

use bitflags::bitflags;
use insim_core::{
    binrw::{self, binrw, BinRead, BinResult},
    duration::{binrw_parse_duration, binrw_write_duration},
};

use crate::identifiers::{PlayerId, RequestId};

#[binrw::parser(reader, endian)]
pub(crate) fn binrw_parse_spclose_strip_reserved_bits() -> BinResult<u16> {
    let res = u16::read_options(reader, endian, ())?;
    // strip the top 4 bits off
    Ok(res & !61440)
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

#[binrw]
#[derive(Debug, Clone, Default)]
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn ensure_high_bits_stripped() {
        assert_eq!(
            binrw_parse_spclose_strip_reserved_bits(
                &mut Cursor::new(61441_u16.to_le_bytes()),
                binrw::Endian::Little,
                ()
            )
            .unwrap(),
            1
        );

        assert_eq!(
            binrw_parse_spclose_strip_reserved_bits(
                &mut Cursor::new(63495_u16.to_le_bytes()),
                binrw::Endian::Little,
                ()
            )
            .unwrap(),
            2055
        );
    }
}
