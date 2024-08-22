//! Utilities for working with various strings from Insim.

use bytes::BufMut;

pub mod codepages;
pub mod colours;
mod control;
pub mod escaping;

/// Strip any trailing \0 bytes from a u8 slice.
pub fn strip_trailing_nul(input: &[u8]) -> &[u8] {
    if let Some(pos) = input.iter().position(|x| *x == 0) {
        &input[..pos]
    } else {
        input
    }
}

use binrw::{helpers::until_eof, BinRead, BinWrite};

#[allow(missing_docs)]
#[binrw::writer(writer, endian)]
pub fn binrw_write_codepage_string<const SIZE: usize>(
    input: &String,
    raw: bool,
    align_to: u8,
) -> binrw::BinResult<()> {
    let mut res: Vec<u8> = if raw {
        input.as_bytes().to_vec()
    } else {
        codepages::to_lossy_bytes(input).to_vec()
    };

    if align_to > 1 {
        let align_to = (align_to as usize) - 1;
        let round_to = (res.len() + align_to) & !align_to;

        if round_to != res.len() {
            res.put_bytes(0, round_to - res.len());
        }

        res.truncate(SIZE);
    } else {
        res.truncate(SIZE);

        let remaining = SIZE - res.len();
        if remaining > 0 {
            res.put_bytes(0, remaining);
        }
    }

    res.write_options(writer, endian, ())?;

    Ok(())
}

#[allow(missing_docs)]
#[binrw::parser(reader, endian)]
pub fn binrw_parse_codepage_string<const SIZE: usize>(raw: bool) -> binrw::BinResult<String> {
    <[u8; SIZE]>::read_options(reader, endian, ()).map(|bytes| {
        let bytes = strip_trailing_nul(&bytes);

        if raw {
            Ok(String::from_utf8_lossy(bytes).to_string())
        } else {
            Ok(codepages::to_lossy_string(bytes).to_string())
        }
    })?
}

#[allow(missing_docs)]
#[binrw::parser(reader, endian)]
pub fn binrw_parse_codepage_string_until_eof(raw: bool) -> binrw::BinResult<String> {
    until_eof(reader, endian, ()).map(|bytes: Vec<u8>| {
        let bytes = strip_trailing_nul(&bytes);
        if raw {
            Ok(String::from_utf8_lossy(bytes).to_string())
        } else {
            Ok(codepages::to_lossy_string(bytes).to_string())
        }
    })?
}
