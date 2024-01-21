//! Utility methods for encoding and decoding various durations and intervals used within Insim.
//! These 2 methods allow the milliseconds to be scaled using the const SCALE generic.
//! This is useful as a number of LFS packets use centiseconds (1/100th of a second, rather than
//! 1/1000th).

use std::time::Duration;

use binrw::{BinRead, BinWrite, Error as BinError};

#[binrw::writer(writer, endian)]
pub fn binrw_write_duration<
    T: TryFrom<u128> + for<'a> BinWrite<Args<'a> = ()>,
    const SCALE: u128,
>(
    input: &Duration,
) -> binrw::BinResult<()> {
    let pos = writer.stream_position()?;

    match T::try_from(input.as_millis() / SCALE) {
        Ok(v) => v.write_options(writer, endian, ()),
        Err(_) => Err(BinError::AssertFail {
            pos,
            message: "Could not convert to duration without loss".into(),
        }),
    }
}

#[binrw::parser(reader, endian)]
pub fn binrw_parse_duration<T: TryInto<u64> + for<'a> BinRead<Args<'a> = ()>, const SCALE: u64>(
) -> binrw::BinResult<Duration> {
    let pos = reader.stream_position()?;
    let res = T::read_options(reader, endian, ())?;
    match TryInto::<u64>::try_into(res) {
        Ok(v) => Ok(Duration::from_millis(v * SCALE)),
        Err(_) => Err(BinError::AssertFail {
            pos,
            message: "Could not convert to duration without loss".into(),
        }),
    }
}
