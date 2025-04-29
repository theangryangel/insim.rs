#![doc = include_str!("../README.md")]

pub mod game_version;
pub mod license;
pub mod point;
pub mod string;
pub mod track;
pub mod vehicle;
pub mod wind;

use std::{array::from_fn, fmt::Display, net::Ipv4Addr, num::TryFromIntError};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use game_version::GameVersionParseError;

// FIXME: rename, add line/contextual information, split into ReadBufError and WriteBufError
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
    /// Value too large for field
    TooLarge,
    /// Game Version Parse Error
    GameVersionParseError(GameVersionParseError),
}

impl std::error::Error for Error {}

impl From<GameVersionParseError> for Error {
    fn from(value: GameVersionParseError) -> Self {
        Self::GameVersionParseError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) // FIXME
    }
}

/// Read from bytes
pub trait ReadWriteBuf: Sized {
    /// Read
    fn read_buf(buf: &mut Bytes) -> Result<Self, Error>;

    /// Write
    fn write_buf(&self, buf: &mut BytesMut) -> Result<(), Error>;
}

impl ReadWriteBuf for char {
    fn read_buf(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u8() as char)
    }

    fn write_buf(&self, buf: &mut BytesMut) -> Result<(), Error> {
        if self.is_ascii() {
            buf.put_u8(*self as u8);
            return Ok(());
        }

        Err(Error::NotAsciiChar { found: *self })
    }
}

impl ReadWriteBuf for bool {
    fn read_buf(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u8() > 0)
    }

    fn write_buf(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_u8(*self as u8);
        Ok(())
    }
}

impl ReadWriteBuf for u8 {
    fn read_buf(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u8())
    }

    fn write_buf(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_u8(*self);

        Ok(())
    }
}

impl ReadWriteBuf for u16 {
    fn read_buf(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u16_le())
    }

    fn write_buf(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_u16_le(*self);

        Ok(())
    }
}

impl ReadWriteBuf for i16 {
    fn read_buf(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_i16_le())
    }

    fn write_buf(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_i16_le(*self);

        Ok(())
    }
}

impl ReadWriteBuf for u32 {
    fn read_buf(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u32_le())
    }

    fn write_buf(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_u32_le(*self);

        Ok(())
    }
}

impl ReadWriteBuf for i32 {
    fn read_buf(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_i32_le())
    }

    fn write_buf(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_i32_le(*self);

        Ok(())
    }
}

impl ReadWriteBuf for f32 {
    fn read_buf(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_f32_le())
    }

    fn write_buf(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_f32_le(*self);

        Ok(())
    }
}

/// Read from bytes
pub trait FromToAsciiBytes: Sized {
    /// Read
    fn from_ascii_bytes(buf: &mut Bytes, len: usize) -> Result<Self, Error>;

    /// Write
    fn to_ascii_bytes(
        &self,
        buf: &mut BytesMut,
        len: usize,
        trailing_nul: bool,
    ) -> Result<(), Error>;
}

impl FromToAsciiBytes for String {
    fn from_ascii_bytes(buf: &mut Bytes, len: usize) -> Result<Self, Error> {
        let new = buf.split_to(len);
        let bytes = string::strip_trailing_nul(&new);
        Ok(String::from_utf8_lossy(bytes).to_string())
    }

    fn to_ascii_bytes(
        &self,
        buf: &mut BytesMut,
        len: usize,
        trailing_nul: bool,
    ) -> Result<(), Error> {
        if !self.is_ascii() {
            return Err(Error::NotAsciiString);
        }
        let new = self.as_bytes();
        let max_len = if trailing_nul { len - 1 } else { len };
        if new.len() > max_len {
            return Err(Error::TooLarge);
        }
        let len_to_write = new.len().min(max_len);
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
    fn to_codepage_bytes(
        &self,
        buf: &mut BytesMut,
        len: usize,
        trailing_nul: bool,
    ) -> Result<(), Error>;

    /// Write variable length, upto len, aligned to nearest X bytes
    fn to_codepage_bytes_aligned(
        &self,
        buf: &mut BytesMut,
        len: usize,
        alignment: usize,
        trailing_nul: bool,
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

    fn to_codepage_bytes(
        &self,
        buf: &mut BytesMut,
        len: usize,
        trailing_nul: bool,
    ) -> Result<(), Error> {
        let new = string::codepages::to_lossy_bytes(self.as_ref());
        let max_len = if trailing_nul { len - 1 } else { len };
        if new.len() > max_len {
            return Err(Error::TooLarge);
        }
        let len_to_write = new.len().min(max_len);

        buf.extend_from_slice(&new[..len_to_write]);
        buf.put_bytes(0, len - len_to_write);
        Ok(())
    }

    fn to_codepage_bytes_aligned(
        &self,
        buf: &mut BytesMut,
        len: usize,
        alignment: usize,
        trailing_nul: bool,
    ) -> Result<(), Error> {
        let new = string::codepages::to_lossy_bytes(self.as_ref());
        let max_len = if trailing_nul { len - 1 } else { len };
        if new.len() > max_len {
            return Err(Error::TooLarge);
        }
        let len_to_write = new.len().min(max_len);
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

impl<T, const N: usize> ReadWriteBuf for [T; N]
where
    T: ReadWriteBuf,
{
    fn read_buf(buf: &mut Bytes) -> Result<Self, Error> {
        let val = from_fn(|_| T::read_buf(buf).unwrap());

        Ok(val)
    }

    fn write_buf(&self, buf: &mut BytesMut) -> Result<(), Error> {
        for i in self.iter() {
            i.write_buf(buf)?;
        }

        Ok(())
    }
}

impl ReadWriteBuf for Ipv4Addr {
    fn read_buf(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(Ipv4Addr::from(u32::read_buf(buf)?))
    }

    fn write_buf(&self, buf: &mut BytesMut) -> Result<(), Error> {
        let repr = u32::from(*self);
        repr.write_buf(buf)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_string_codepage_bytes_aligned_abc() {
        let input = String::from("abc");
        let mut buf = BytesMut::new();
        input
            .to_codepage_bytes_aligned(&mut buf, 128, 4, false)
            .unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', b'b', b'c', 0]);
    }

    #[test]
    fn test_string_codepage_bytes_aligned_abcd() {
        let input = String::from("abcd");
        let mut buf = BytesMut::new();
        input
            .to_codepage_bytes_aligned(&mut buf, 128, 4, false)
            .unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', b'b', b'c', b'd']);
    }

    #[test]
    fn test_string_codepage_bytes_trialing_nul() {
        let input = String::from("a");
        let mut buf = BytesMut::new();
        input
            .to_codepage_bytes_aligned(&mut buf, 2, 2, true)
            .unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', 0]);
    }

    #[test]
    fn test_string_ascii_bytes_trialing_nul() {
        let input = String::from("ab");
        let mut buf = BytesMut::new();
        input.to_ascii_bytes(&mut buf, 4, true).unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', b'b', 0, 0]);
    }

    #[test]
    fn test_string_ascii_too_long() {
        let input = String::from("abcde");
        let mut buf = BytesMut::new();
        let res = input.to_ascii_bytes(&mut buf, 4, true);
        assert!(res.is_err());
    }

    #[test]
    fn test_string_codepage_too_long() {
        let input = String::from("abcde");
        let mut buf = BytesMut::new();
        let res = input.to_codepage_bytes(&mut buf, 4, true);
        assert!(res.is_err());
    }
}
