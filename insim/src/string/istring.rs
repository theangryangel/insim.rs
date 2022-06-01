//! Utility functions for working with LFS' CString-like strings. If you need formatting through
//! Codepages you should be looking at the ICodePageString custom struct.
//!
//! Effectively LFS transmits strings as a "CString", with the exception that a CString must always
//! be terminated by a \0 byte. In LFS's wireformat this is not always the case.
//!
//! The istring module provides simple methods for reading and writing these.

use super::strip_trailing_nul;
use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::{ctx::*, DekuError, DekuRead, DekuWrite};
use std::vec::Vec;

/// Parse a simple optionally \0 terminated string from a byte array into a String.
pub fn read(
    rest: &BitSlice<Msb0, u8>,
    bytes: usize,
) -> Result<(&BitSlice<Msb0, u8>, String), DekuError> {
    let size = Size::Bytes(bytes);
    let (rest, value) = Vec::read(rest, Limit::new_size(size))?;

    let value = strip_trailing_nul(&value);

    Ok((
        rest,
        String::from_utf8(value.to_vec()).map_err(|e| DekuError::Parse(e.to_string()))?,
    ))
}

/// Write a simple non-codepage/non-format-able String into LFS's optionally \0 terminated wire format.
pub fn write(output: &mut BitVec<Msb0, u8>, field: &str, bytes: usize) -> Result<(), DekuError> {
    let size = Size::Bytes(bytes);

    let input = field.as_bytes();
    let orig_size = output.len();

    if input.is_empty() {
        output.resize(orig_size + size.bit_size(), false);
        return Ok(());
    }

    let max_size = size.byte_size()?;
    let input_size = if input.len() < max_size {
        input.len()
    } else {
        max_size
    };

    let res = (&input[0..input_size]).write(output, ());
    if let Err(e) = res {
        return Err(e);
    }

    if input_size != max_size {
        output.resize(orig_size + size.bit_size(), false);
    }

    Ok(())
}
