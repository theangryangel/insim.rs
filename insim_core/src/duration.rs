use std::time::Duration;

use binrw::{BinRead, BinWrite, Error as BinError};

// FIXME binrw_parse_duration<T> should be possible

#[binrw::writer(writer, endian)]
pub fn binrw_write_duration<T: TryFrom<u128> + for<'a> BinWrite<Args<'a> = ()>>(
    input: &Duration,
) -> binrw::BinResult<()> {
    let pos = writer.stream_position()?;

    match T::try_from(input.as_millis()) {
        Ok(v) => v.write_options(writer, endian, ()),
        Err(_) => Err(BinError::AssertFail {
            pos,
            message: "Could not convert to duration without loss".into(),
        }),
    }
}

#[binrw::parser(reader, endian)]
pub fn binrw_parse_duration<T: TryInto<u64> + for<'a> BinRead<Args<'a> = ()>>(
) -> binrw::BinResult<Duration> {
    let pos = reader.stream_position()?;
    let res = T::read_options(reader, endian, ())?;
    match TryInto::<u64>::try_into(res) {
        Ok(v) => Ok(Duration::from_millis(v)),
        Err(_) => Err(BinError::AssertFail {
            pos,
            message: "Could not convert to duration without loss".into(),
        }),
    }
}
