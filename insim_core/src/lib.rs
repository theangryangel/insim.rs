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
