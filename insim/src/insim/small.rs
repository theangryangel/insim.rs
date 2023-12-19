use std::time::Duration;

use bitflags::bitflags;

use insim_core::{
    binrw::{self, binrw, BinRead, BinWrite},
    identifiers::RequestId,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::VtnAction;

bitflags! {
    /// Bitwise flags used within the [SmallType] packet, Lcs
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct LcsFlags: u32 {
        const SET_SIGNALS = (1 << 0);
        const SET_FLASH = (1 << 1);
        const SET_HEADLIGHTS = (1 << 2);
        const SET_HORN = (1 << 3);
        const SET_SIREN = (1 << 4);

        const SIGNAL_OFF = Self::SET_SIGNALS.bits();
        const SIGNAL_LEFT = Self::SET_SIGNALS.bits() | (1 << 8);
        const SIGNAL_RIGHT = Self::SET_SIGNALS.bits() | (2 << 8);
        const SIGNAL_HAZARD = Self::SET_SIGNALS.bits() | (3 << 8);

        const FLASH_OFF = Self::SET_FLASH.bits();
        const FLASH_ON = Self::SET_FLASH.bits() | (1 << 10);

        const HEADLIGHTS_OFF = Self::SET_HEADLIGHTS.bits();
        const HEADLIGHTS_ON = Self::SET_HEADLIGHTS.bits() | (1 << 11);

        const HORN_OFF = Self::SET_HORN.bits();
        const HORN_1 = Self::SET_HORN.bits() | (1 << 16);
        const HORN_2 = Self::SET_HORN.bits() | (2 << 16);
        const HORN_3 = Self::SET_HORN.bits() | (3 << 16);
        const HORN_4 = Self::SET_HORN.bits() | (4 << 16);
        const HORN_5 = Self::SET_HORN.bits() | (5 << 16);

        const SIREN_OFF = Self::SET_SIREN.bits();
        const SIREN_FAST = Self::SET_SIREN.bits() | (1 << 20);
        const SIREN_SLOW = Self::SET_SIREN.bits() | (2 << 20);
    }
}

bitflags! {
    /// Bitwise flags used within the [SmallType] packet, Lcl
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct LclFlags: u32 {
        const SET_SIGNALS = (1 << 0);
        const SET_LIGHTS = (1 << 2);
        const SET_FOG_REAR = (1 << 4);
        const SET_FOG_FRONT = (1 << 5);
        const SET_EXTRA = (1 << 6);

        const SIGNAL_OFF = Self::SET_SIGNALS.bits();
        const SIGNAL_LEFT = Self::SET_SIGNALS.bits() | (1 << 16);
        const SIGNAL_RIGHT = Self::SET_SIGNALS.bits() | (2 << 16);
        const SIGNAL_HAZARD = Self::SET_SIGNALS.bits() | (3 << 16);

        const LIGHT_OFF = Self::SET_LIGHTS.bits();
        const LIGHT_SIDE = Self::SET_LIGHTS.bits() | (1 << 18);
        const LIGHT_LOW = Self::SET_LIGHTS.bits() | (2 << 18);
        const LIGHT_HIGH = Self::SET_LIGHTS.bits() | (3 << 18);

        const FOG_REAR_OFF = Self::SET_FOG_REAR.bits();
        const FOG_REAR = Self::SET_FOG_REAR.bits() | (1 << 20);

        const FOG_FRONT_OFF = Self::SET_FOG_FRONT.bits();
        const FOG_FRONT = Self::SET_FOG_FRONT.bits() | (1 << 21);

        const EXTRA_OFF = Self::SET_EXTRA.bits();
        const EXTRA = Self::SET_EXTRA.bits() | (1 << 2);
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum SmallType {
    None,

    /// Request LFS to start sending positions
    Ssp(Duration),

    /// Request LFS to start sending gauges
    Ssg(Duration),

    /// Vote action
    Vta(VtnAction),

    /// Time stop
    Tms(bool),

    /// Time step
    Stp(Duration),

    /// Race time packet (reply to Gth)
    Rtp(Duration),

    /// Set node lap interval
    Nli(Duration),

    /// Set or get allowed cars (Tiny, type = Alc)
    Alc(u32),

    /// Set local car switches
    Lcs(LcsFlags),

    /// Set local vehicle lights
    Lcl(LclFlags),
}

impl Default for SmallType {
    fn default() -> Self {
        Self::None
    }
}

impl BinRead for SmallType {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let pos = reader.stream_position()?;
        let discrim = u8::read_options(reader, endian, ())?;
        let uval = u32::read_options(reader, endian, ())?;
        let res = match discrim {
            0 => Self::None,
            1 => Self::Ssp(Duration::from_millis(uval as u64 * 10)),
            2 => Self::Ssg(Duration::from_millis(uval as u64 * 10)),
            3 => Self::Vta(uval.into()),
            4 => Self::Tms(uval != 0),
            5 => Self::Stp(Duration::from_millis(uval as u64 * 10)),
            6 => Self::Rtp(Duration::from_millis(uval as u64 * 10)),
            7 => Self::Nli(Duration::from_millis(uval as u64)),
            8 => Self::Alc(uval),
            9 => Self::Lcs(LcsFlags::from_bits_truncate(uval)),
            10 => Self::Lcl(LclFlags::from_bits_truncate(uval)),
            _ => {
                return Err(binrw::Error::BadMagic {
                    pos,
                    found: Box::new(uval),
                })
            }
        };
        Ok(res)
    }
}

impl BinWrite for SmallType {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        let (discrim, uval) = match self {
            SmallType::None => (0u8, 0u32),
            SmallType::Ssp(uval) => (1u8, uval.as_millis() as u32 / 10),
            SmallType::Ssg(uval) => (2u8, uval.as_millis() as u32 / 10),
            SmallType::Vta(uval) => (3u8, uval.into()),
            SmallType::Tms(uval) => (4u8, *uval as u32),
            SmallType::Stp(uval) => (5u8, uval.as_millis() as u32 / 10),
            SmallType::Rtp(uval) => (6u8, uval.as_millis() as u32 / 10),
            SmallType::Nli(uval) => (7u8, uval.as_millis() as u32),
            SmallType::Alc(_) => todo!(),
            SmallType::Lcs(uval) => (9u8, uval.bits()),
            SmallType::Lcl(uval) => (10u8, uval.bits()),
        };

        discrim.write_options(writer, endian, ())?;
        uval.write_options(writer, endian, ())?;

        Ok(())
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// General purpose Small packet
pub struct Small {
    pub reqi: RequestId,

    pub subt: SmallType,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_small_none() {
        let data = Small {
            reqi: RequestId(1),
            subt: SmallType::None,
        };

        let mut writer = Cursor::new(Vec::new());
        data.write_le(&mut writer).unwrap();
        let buf = writer.into_inner();

        assert_eq!(buf.len(), 6);
        assert_eq!(buf, [1, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_small_ssp() {
        let data = Small {
            reqi: RequestId(1),
            subt: SmallType::Ssp(Duration::from_secs(1)),
        };

        let mut writer = Cursor::new(Vec::new());
        data.write_le(&mut writer).unwrap();
        let buf = writer.into_inner();

        assert_eq!(buf, [1, 1, 100, 0, 0, 0]);
    }
}
