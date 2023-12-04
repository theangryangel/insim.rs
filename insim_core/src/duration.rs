use std::time::Duration;

use binrw::{BinRead, BinWrite};

// FIXME binrw_parse_duration<T> should be possible

#[binrw::writer(writer, endian)]
pub fn binrw_write_u32_duration(input: &Duration) -> binrw::BinResult<()> {
    let res = input.as_millis() as u32;
    res.write_options(writer, endian, ())
}

#[binrw::parser(reader, endian)]
pub fn binrw_parse_u32_duration() -> binrw::BinResult<Duration> {
    let res = u32::read_options(reader, endian, ())?;
    Ok(Duration::from_millis(res as u64))
}

#[binrw::writer(writer, endian)]
pub fn binrw_write_u16_duration(input: &Duration) -> binrw::BinResult<()> {
    let res = input.as_millis() as u16;
    res.write_options(writer, endian, ())
}

#[binrw::parser(reader, endian)]
pub fn binrw_parse_u16_duration() -> binrw::BinResult<Duration> {
    let res = u16::read_options(reader, endian, ())?;
    Ok(Duration::from_millis(res as u64))
}
