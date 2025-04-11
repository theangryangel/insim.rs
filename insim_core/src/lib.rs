#![doc = include_str!("../README.md")]

pub mod duration;
pub mod game_version;
pub mod license;
pub mod point;
pub mod string;
pub mod track;
pub mod vehicle;
pub mod wind;

use std::{array::from_fn, num::TryFromIntError};

#[doc(hidden)]
pub use ::binrw;
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[non_exhaustive]
/// Read/Write Error
#[derive(Debug)]
pub enum Error {
    /// Bad Magic
    BadMagic {
        /// found
        found: Box<dyn core::fmt::Debug + Send + Sync>,
    },
    /// No Variant
    NoVariantMatch {
        /// found
        found: u64,
    },
    /// Cannot convert
    NotAsciiChar {
        /// Found character
        found: char,
    },
    /// String is not completely Ascii
    NotAsciiString,
    /// TryFromInt
    TryFromInt(TryFromIntError),
    /// Duration too large for packet
    DurationTooLarge,
}

/// Read from bytes
pub trait FromToBytes: Sized {
    /// Read
    fn from_bytes(buf: &mut Bytes) -> Result<Self, Error>;

    /// Write
    fn to_bytes(&self, buf: &mut BytesMut) -> Result<(), Error>;
}

impl FromToBytes for char {
    fn from_bytes(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u8() as char)
    }

    fn to_bytes(&self, buf: &mut BytesMut) -> Result<(), Error> {
        if self.is_ascii() {
            buf.put_u8(*self as u8);
            return Ok(());
        }

        Err(Error::NotAsciiChar { found: *self })
    }
}

impl FromToBytes for u8 {
    fn from_bytes(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u8())
    }

    fn to_bytes(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_u8(*self);

        Ok(())
    }
}

impl FromToBytes for u16 {
    fn from_bytes(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u16_le())
    }

    fn to_bytes(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_u16_le(*self);

        Ok(())
    }
}

impl FromToBytes for i16 {
    fn from_bytes(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_i16_le())
    }

    fn to_bytes(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_i16_le(*self);

        Ok(())
    }
}

impl FromToBytes for u32 {
    fn from_bytes(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u32_le())
    }

    fn to_bytes(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_u32_le(*self);

        Ok(())
    }
}

impl FromToBytes for i32 {
    fn from_bytes(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_i32_le())
    }

    fn to_bytes(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_i32_le(*self);

        Ok(())
    }
}

impl FromToBytes for f32 {
    fn from_bytes(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_f32_le())
    }

    fn to_bytes(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_f32_le(*self);

        Ok(())
    }
}

/// Read from bytes
pub trait FromToAsciiBytes: Sized {
    /// Read
    fn from_ascii_bytes(buf: &mut Bytes, len: usize) -> Result<Self, Error>;

    /// Write
    fn to_ascii_bytes(&self, buf: &mut BytesMut, len: usize) -> Result<(), Error>;
}

impl FromToAsciiBytes for String {
    fn from_ascii_bytes(buf: &mut Bytes, len: usize) -> Result<Self, Error> {
        let new = buf.split_to(len);
        let bytes = string::strip_trailing_nul(&new);
        Ok(String::from_utf8_lossy(bytes).to_string())
    }

    fn to_ascii_bytes(&self, buf: &mut BytesMut, len: usize) -> Result<(), Error> {
        if !self.is_ascii() {
            return Err(Error::NotAsciiString);
        }
        let new = self.as_bytes();
        let len_to_write = new.len().min(len);
        buf.extend_from_slice(&new[..len_to_write]);
        buf.put_bytes(0, len - len_to_write);
        Ok(())
    }
}

/// Read from bytes
pub trait FromToCodepageBytes {
    /// Read
    fn from_codepage_bytes(buf: &mut Bytes, len: usize) -> Result<String, Error>;

    /// Write fixed length
    fn to_codepage_bytes(&self, buf: &mut BytesMut, len: usize) -> Result<(), Error>;

    /// Write variable length, upto len, aligned to nearest X bytes
    fn to_codepage_bytes_aligned(
        &self,
        buf: &mut BytesMut,
        len: usize,
        alignment: usize,
    ) -> Result<(), Error>;
}

impl<T> FromToCodepageBytes for T
where
    T: AsRef<str>,
{
    fn from_codepage_bytes(buf: &mut Bytes, len: usize) -> Result<String, Error> {
        let new = buf.split_to(buf.len().min(len));
        let new = string::codepages::to_lossy_string(string::strip_trailing_nul(&new));
        Ok(new.to_string())
    }

    fn to_codepage_bytes(&self, buf: &mut BytesMut, len: usize) -> Result<(), Error> {
        let new = string::codepages::to_lossy_bytes(self.as_ref());
        let len_to_write = new.len().min(len);
        buf.extend_from_slice(&new[..len_to_write]);
        buf.put_bytes(0, len - len_to_write);
        Ok(())
    }

    fn to_codepage_bytes_aligned(
        &self,
        buf: &mut BytesMut,
        len: usize,
        alignment: usize,
    ) -> Result<(), Error> {
        let new = string::codepages::to_lossy_bytes(self.as_ref());
        let len_to_write = new.len().min(len);
        buf.extend_from_slice(&new[..len_to_write]);
        if len_to_write < len {
            let align_to = alignment - 1;
            let round_to = (len_to_write + align_to) & !align_to;
            let round_to = round_to.min(len);
            buf.put_bytes(0, round_to - len_to_write);
        }
        Ok(())
    }
}

impl<T, const N: usize> FromToBytes for [T; N]
where
    T: FromToBytes,
{
    fn from_bytes(buf: &mut Bytes) -> Result<Self, Error> {
        let val = from_fn(|_| T::from_bytes(buf).unwrap());

        Ok(val)
    }

    fn to_bytes(&self, buf: &mut BytesMut) -> Result<(), Error> {
        for i in self.iter() {
            i.to_bytes(buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_string_codepage_bytes_aligned_abc() {
        let input = String::from("abc");
        let mut buf = BytesMut::new();
        input.to_codepage_bytes_aligned(&mut buf, 128, 4).unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', b'b', b'c', 0]);
    }

    #[test]
    fn test_string_codepage_bytes_aligned_abcd() {
        let input = String::from("abcd");
        let mut buf = BytesMut::new();
        input.to_codepage_bytes_aligned(&mut buf, 128, 4).unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', b'b', b'c', b'd']);
    }

    #[test]
    fn test_string_codepage_bytes_aligned_truncates() {
        let input = String::from("abcd");
        let mut buf = BytesMut::new();
        input.to_codepage_bytes_aligned(&mut buf, 2, 2).unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', b'b']);
    }
}
