//! Decode trait

use std::net::Ipv4Addr;

use bytes::{Buf, Bytes};

/// DecodeError
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    /// Bad Magic
    #[error("Bad magic. Found: {:?}", found)]
    BadMagic {
        /// found
        found: Box<dyn core::fmt::Debug + Send + Sync>,
    },

    #[error("no variant match: {:?}", found)]
    /// No Variant
    NoVariantMatch {
        /// found
        found: u64,
    },

    /// Game Version Parse Error
    #[error("could not parse game version: {0}")]
    GameVersionParseError(#[from] crate::game_version::GameVersionParseError),

    /// Value too large for field
    #[error("too large")]
    TooLarge,

    /// Expected \0 character
    #[error("Expected \0 character")]
    ExpectedNull,
}

/// Decode from bytes
pub trait Decode: Sized {
    /// Read
    fn decode(buf: &mut Bytes) -> Result<Self, DecodeError>;
}

/// Read from bytes
pub trait DecodeString: Sized {
    /// Read bytes as Ascii
    fn decode_ascii(buf: &mut Bytes, len: usize) -> Result<Self, DecodeError>;

    /// Read bytes as codepage encoded
    fn decode_codepage(buf: &mut Bytes, len: usize) -> Result<String, DecodeError>;
}

// impls

impl Decode for char {
    fn decode(buf: &mut Bytes) -> Result<Self, DecodeError> {
        Ok(buf.get_u8() as char)
    }
}

impl Decode for bool {
    fn decode(buf: &mut Bytes) -> Result<Self, DecodeError> {
        Ok(buf.get_u8() > 0)
    }
}

impl Decode for u8 {
    fn decode(buf: &mut Bytes) -> Result<Self, DecodeError> {
        Ok(buf.get_u8())
    }
}

impl Decode for u16 {
    fn decode(buf: &mut Bytes) -> Result<Self, DecodeError> {
        Ok(buf.get_u16_le())
    }
}

impl Decode for i16 {
    fn decode(buf: &mut Bytes) -> Result<Self, DecodeError> {
        Ok(buf.get_i16_le())
    }
}

impl Decode for u32 {
    fn decode(buf: &mut Bytes) -> Result<Self, DecodeError> {
        Ok(buf.get_u32_le())
    }
}

impl Decode for i32 {
    fn decode(buf: &mut Bytes) -> Result<Self, DecodeError> {
        Ok(buf.get_i32_le())
    }
}

impl Decode for f32 {
    fn decode(buf: &mut Bytes) -> Result<Self, DecodeError> {
        Ok(buf.get_f32_le())
    }
}

impl<T, const N: usize> Decode for [T; N]
where
    T: Decode,
{
    fn decode(buf: &mut Bytes) -> Result<Self, DecodeError> {
        // If T::decode returns Err(BadMagic), collect stops and returns Err(BadMagic) immediately.
        let items: Vec<T> = (0..N)
            .map(|_| T::decode(buf))
            .collect::<Result<Vec<T>, DecodeError>>()?;

        // We use .ok().expect() because if this fails, it's a bug in the code,
        // not a problem with the input data.
        Ok(items
            .try_into()
            .ok()
            .expect("size must match N because we looped N times"))
    }
}

impl Decode for Ipv4Addr {
    fn decode(buf: &mut Bytes) -> Result<Self, DecodeError> {
        Ok(Ipv4Addr::from(u32::decode(buf)?))
    }
}

impl DecodeString for String {
    fn decode_ascii(buf: &mut Bytes, len: usize) -> Result<Self, DecodeError> {
        let new = buf.split_to(len);
        let bytes = crate::string::strip_trailing_nul(&new);
        Ok(String::from_utf8_lossy(bytes).to_string())
    }

    fn decode_codepage(buf: &mut Bytes, len: usize) -> Result<String, DecodeError> {
        let new = buf.split_to(buf.len().min(len));
        let new =
            crate::string::codepages::to_lossy_string(crate::string::strip_trailing_nul(&new));
        Ok(new.to_string())
    }
}
