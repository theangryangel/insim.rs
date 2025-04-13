use std::{
    io::{Read, Seek, Write},
    time::Duration,
};

use insim_core::{
    binrw::{self, binrw, BinRead, BinResult, BinWrite, Endian},
    duration::{binrw_parse_duration, binrw_write_duration},
    ReadWriteBuf,
};

use super::{PenaltyInfo, PlayerFlags};
use crate::identifiers::{PlayerId, RequestId};

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
/// When /showfuel yes: double fuel percent / no: 255
pub enum Fuel200 {
    /// Double fuel percent
    Percentage(u8),

    /// Fuel cannot be reported, /showfuel=no
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
            Self::No => 255_u8,
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

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
/// When /showfuel yes: fuel added percent / no: 255
pub enum Fuel {
    /// Double fuel percent
    Percentage(u8),

    /// Fuel cannot be reported, /showfuel=no
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
            Self::No => 255_u8,
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

impl ReadWriteBuf for Fuel {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let data = u8::read_buf(buf)?;
        if data == 255 {
            Ok(Self::No)
        } else {
            Ok(Self::Percentage(data))
        }
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        let data = match self {
            Self::Percentage(data) => *data,
            Self::No => 255_u8,
        };
        data.write_buf(buf)?;
        Ok(())
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Lap Time for a given player.
pub struct Lap {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player ID
    pub plid: PlayerId,

    #[br(parse_with = binrw_parse_duration::<u32, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 1, _>)]
    /// Lap time
    pub ltime: Duration, // lap time (ms)

    #[br(parse_with = binrw_parse_duration::<u32, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 1, _>)]
    /// Total elapsed time
    pub etime: Duration,

    /// Number of laps completed.
    pub lapsdone: u16,

    /// See [PlayerFlags].
    #[brw(pad_after = 1)]
    pub flags: PlayerFlags,

    /// Current penalty
    pub penalty: PenaltyInfo,

    /// Number of pit stops.
    pub numstops: u8,

    /// See [Fuel200].
    pub fuel200: Fuel200,
}
