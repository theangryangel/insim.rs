use std::time::Duration;

use insim_core::{
    binrw::{self, binrw, BinRead, BinWrite},
    identifiers::RequestId,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::VtnAction;

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
    Lcs(u32),

    /// Set local vehicle lights
    Lcl(u32),
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
            9 => Self::Lcs(uval),
            10 => Self::Lcl(uval),
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
            SmallType::Lcs(_) => todo!(),
            SmallType::Lcl(_) => todo!(),
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
