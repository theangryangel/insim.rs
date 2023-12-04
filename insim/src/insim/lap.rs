use std::{
    io::{Read, Seek, Write},
    time::Duration,
};

use insim_core::{
    binrw::{self, binrw, BinRead, BinResult, BinWrite, Endian},
    duration::{binrw_parse_u32_duration, binrw_write_u32_duration},
    identifiers::{PlayerId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::{PenaltyInfo, PlayerFlags};

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Default, Clone)]
pub enum Fuel200 {
    Percentage(u8),

    #[default]
    No,
}

impl BinWrite for Fuel200 {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        let data = match self {
            Self::Percentage(data) => *data,
            Self::No => 255 as u8,
        };

        data.write_options(writer, endian, args)?;
        Ok(())
    }
}

impl BinRead for Fuel200 {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let data = <u8>::read_options(reader, endian, ())?;

        if data == 255 {
            Ok(Self::No)
        } else {
            Ok(Self::Percentage(data))
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Default, Clone)]
pub enum Fuel {
    Percentage(u8),

    #[default]
    No,
}

impl BinWrite for Fuel {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        let data = match self {
            Self::Percentage(data) => *data,
            Self::No => 255 as u8,
        };

        data.write_options(writer, endian, args)?;
        Ok(())
    }
}

impl BinRead for Fuel {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let data = <u8>::read_options(reader, endian, ())?;

        if data == 255 {
            Ok(Self::No)
        } else {
            Ok(Self::Percentage(data))
        }
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Lap Time for a given player.
pub struct Lap {
    pub reqi: RequestId,
    pub plid: PlayerId,

    #[br(parse_with = binrw_parse_u32_duration::<_>)]
    #[bw(write_with = binrw_write_u32_duration::<_>)]
    pub ltime: Duration, // lap time (ms)

    #[br(parse_with = binrw_parse_u32_duration::<_>)]
    #[bw(write_with = binrw_write_u32_duration::<_>)]
    pub etime: Duration,

    pub lapsdone: u16,
    #[brw(pad_after = 1)]
    pub flags: PlayerFlags,

    pub penalty: PenaltyInfo,
    pub numstops: u8,
    pub fuel200: Fuel200,
}
