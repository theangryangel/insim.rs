#![doc = include_str!("../README.md")]

pub mod duration;
pub mod game_version;
pub mod license;
pub mod point;
pub mod string;
pub mod track;
pub mod vehicle;
pub mod wind;

use std::num::TryFromIntError;

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
    /// TryFromInt
    TryFromInt(TryFromIntError),
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
pub trait FromToCodepageBytes: Sized {
    /// Read
    fn from_codepage_bytes(buf: &mut Bytes, len: usize) -> Result<Self, Error>;

    /// Write
    fn to_codepage_bytes(&self, buf: &mut BytesMut, len: usize) -> Result<(), Error>;
}

impl FromToCodepageBytes for String {
    fn from_codepage_bytes(buf: &mut Bytes, len: usize) -> Result<Self, Error> {
        let new = buf.split_to(len);
        let new = string::codepages::to_lossy_string(
            string::strip_trailing_nul(&new)
        );
        Ok(new.to_string())
    }

    fn to_codepage_bytes(&self, buf: &mut BytesMut, len: usize) -> Result<(), Error> {
        let new = string::codepages::to_lossy_bytes(&self);
        let len_to_write = new.len().min(len);
        buf.extend_from_slice(&new);
        buf.put_bytes(0, len - len_to_write);
        Ok(())
    }
}

#[macro_export]
#[allow(missing_docs)]
macro_rules! to_bytes_padded {
    ($buf:expr, $data:expr, $size:expr) => {{
        let len = $data.len().min($size);
        $buf.extend_from_slice(&$data[..len]);
        $buf.put_bytes(0, $size - len);
    }};
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;
    use bytes::BufMut;

    #[test]
    fn test_macro_to_bytes_padded() {    
        let data = b"hello";
        let size = 10;
        let mut buf = BytesMut::with_capacity(size);
        
        to_bytes_padded!(buf, data, size);
        
        assert_eq!(&buf[..], b"hello\0\0\0\0\0");
    }
}
