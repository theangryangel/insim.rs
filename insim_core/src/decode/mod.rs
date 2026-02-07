//! Decode trait

use std::{borrow::Cow, net::Ipv4Addr};

use arrayvec::ArrayVec;
use bytes::{Buf, Bytes};

#[derive(Debug, thiserror::Error)]
/// Decoding error
pub struct DecodeError {
    /// Optional contextual information
    pub context: Option<Cow<'static, str>>,
    /// Kind of error
    pub kind: DecodeErrorKind,
}

impl DecodeError {
    /// Add context to this error
    pub fn context(mut self, ctx: impl Into<Cow<'static, str>>) -> Self {
        self.context = Some(ctx.into());
        self
    }

    /// Create a nested error quickly
    pub fn nested(self) -> Self {
        Self {
            context: None,
            kind: DecodeErrorKind::Nested {
                source: Box::new(self),
            },
        }
    }
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut current: Option<&DecodeError> = Some(self);
        let mut first = true;

        while let Some(err) = current {
            if let Some(ctx) = &err.context {
                if !first {
                    f.write_str(" > ")?;
                }
                f.write_str(ctx)?;
                first = false;
            }

            match &err.kind {
                DecodeErrorKind::Nested { source } => current = Some(source),
                kind => {
                    if !first {
                        f.write_str(" > ")?;
                    }
                    write!(f, "{kind}")?;
                    break;
                },
            }
        }
        Ok(())
    }
}

impl From<DecodeErrorKind> for DecodeError {
    fn from(value: DecodeErrorKind) -> Self {
        Self {
            context: None,
            kind: value,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
/// Kind of DecodeError
pub enum DecodeErrorKind {
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

    /// Value too large or small for field
    #[error("Out of valid range")]
    OutOfRange {
        /// Minimum valid size
        min: usize,
        /// Maximum valid size
        max: usize,
        /// found
        found: usize,
    },

    /// Expected \0 character
    #[error("Expected \0 character")]
    ExpectedNull,

    /// Nested error - designed to preserve the full chain of errors
    #[error("{source}")]
    Nested {
        /// Source
        #[source]
        source: Box<DecodeError>,
    },
}

impl DecodeErrorKind {
    /// Add context to this error
    pub fn context(self, ctx: impl Into<Cow<'static, str>>) -> DecodeError {
        DecodeError {
            kind: self,
            context: Some(ctx.into()),
        }
    }
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
        // TODO: waiting for `std::array::try_from_fn` to stablise.
        // For now we'll use ArrayVec to reduce the allocation count that the previous
        // implementation using Vec had.
        // This is a choice based on reducing handling higher frequency packets, such as MCI.

        let mut vec = ArrayVec::<T, N>::new();
        for _ in 0..N {
            vec.push(T::decode(buf)?);
        }
        // expect is safe since we pushed exactly N
        Ok(vec
            .into_inner()
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

#[cfg(test)]
mod test {
    use bytes::Bytes;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    enum Status {
        Active = 1,
        Inactive = 2,
    }

    impl Decode for Status {
        fn decode(buf: &mut Bytes) -> Result<Self, DecodeError> {
            match u8::decode(buf)? {
                1 => Ok(Status::Active),
                2 => Ok(Status::Inactive),
                found => Err(DecodeErrorKind::NoVariantMatch {
                    found: found as u64,
                }
                .into()),
            }
        }
    }

    #[derive(Debug)]
    struct Outer {
        #[allow(unused)]
        status: Status,
    }

    impl Decode for Outer {
        fn decode(buf: &mut Bytes) -> Result<Self, DecodeError> {
            let status = Status::decode(buf).map_err(|e| e.nested().context("Outer::status"))?;
            Ok(Self { status })
        }
    }

    #[test]
    fn test_nested_error_chain() {
        // Invalid status byte (99 doesn't match any variant)
        let mut buf = Bytes::from_static(&[99]);
        let result = Outer::decode(&mut buf);

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Outer::status"));
        assert!(err_msg.contains("no variant match"));
    }
}
