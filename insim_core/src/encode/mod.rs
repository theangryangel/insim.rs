//! Encode trait

use std::net::Ipv4Addr;

use bytes::{BufMut, BytesMut};

#[non_exhaustive]
/// Read/Write Error
#[derive(thiserror::Error, Debug)]
pub enum EncodeError {
    #[error("no variant match: {:?}", found)]
    /// No Variant
    NoVariantMatch {
        /// found
        found: u64,
    },

    /// String is not completely Ascii
    #[error("not an ascii string")]
    NotAsciiString,

    /// Cannot convert
    #[error("not an ascii char: {:?}", found)]
    NotAsciiChar {
        /// Found character
        found: char,
    },

    /// Value too large for field
    #[error("too large")]
    TooLarge,

    /// Value too small for field
    #[error("too small")]
    TooSmall,
}

/// Enable to bytes
pub trait Encode: Sized {
    /// Write
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodeError>;
}

/// Write to bytes
pub trait EncodeString: Sized {
    /// Write
    fn encode_ascii(
        &self,
        buf: &mut BytesMut,
        len: usize,
        trailing_nul: bool,
    ) -> Result<(), EncodeError>;

    /// Write fixed length
    fn encode_codepage(
        &self,
        buf: &mut BytesMut,
        len: usize,
        trailing_nul: bool,
    ) -> Result<(), EncodeError>;

    /// Write variable length, upto len, aligned to nearest X bytes
    fn encode_codepage_with_alignment(
        &self,
        buf: &mut BytesMut,
        len: usize,
        alignment: usize,
        trailing_nul: bool,
    ) -> Result<(), EncodeError>;
}

// impls

impl Encode for char {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodeError> {
        if self.is_ascii() {
            buf.put_u8(*self as u8);
            return Ok(());
        }

        Err(EncodeError::NotAsciiChar { found: *self })
    }
}

impl Encode for bool {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodeError> {
        buf.put_u8(*self as u8);
        Ok(())
    }
}

impl Encode for u8 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodeError> {
        buf.put_u8(*self);

        Ok(())
    }
}

impl Encode for u16 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodeError> {
        buf.put_u16_le(*self);

        Ok(())
    }
}

impl Encode for i16 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodeError> {
        buf.put_i16_le(*self);

        Ok(())
    }
}

impl Encode for u32 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodeError> {
        buf.put_u32_le(*self);

        Ok(())
    }
}

impl Encode for i32 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodeError> {
        buf.put_i32_le(*self);

        Ok(())
    }
}

impl Encode for f32 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodeError> {
        buf.put_f32_le(*self);

        Ok(())
    }
}

impl<T, const N: usize> Encode for [T; N]
where
    T: Encode,
{
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodeError> {
        for i in self.iter() {
            i.encode(buf)?;
        }

        Ok(())
    }
}

impl Encode for Ipv4Addr {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodeError> {
        let repr = u32::from(*self);
        repr.encode(buf)
    }
}

impl<T> EncodeString for T
where
    T: AsRef<str>,
{
    fn encode_ascii(
        &self,
        buf: &mut BytesMut,
        len: usize,
        trailing_nul: bool,
    ) -> Result<(), EncodeError> {
        if !self.as_ref().is_ascii() {
            return Err(EncodeError::NotAsciiString);
        }
        let new = self.as_ref().as_bytes();
        let max_len = if trailing_nul { len - 1 } else { len };
        if new.len() > max_len {
            return Err(EncodeError::TooLarge);
        }
        let len_to_write = new.len().min(max_len);
        buf.extend_from_slice(&new[..len_to_write]);
        buf.put_bytes(0, len - len_to_write);
        Ok(())
    }

    fn encode_codepage(
        &self,
        buf: &mut BytesMut,
        len: usize,
        trailing_nul: bool,
    ) -> Result<(), EncodeError> {
        let new = crate::string::codepages::to_lossy_bytes(self.as_ref());
        let max_len = if trailing_nul { len - 1 } else { len };
        if new.len() > max_len {
            return Err(EncodeError::TooLarge);
        }
        let len_to_write = new.len().min(max_len);

        buf.extend_from_slice(&new[..len_to_write]);
        buf.put_bytes(0, len - len_to_write);
        Ok(())
    }

    fn encode_codepage_with_alignment(
        &self,
        buf: &mut BytesMut,
        len: usize,
        alignment: usize,
        trailing_nul: bool,
    ) -> Result<(), EncodeError> {
        let new = crate::string::codepages::to_lossy_bytes(self.as_ref());
        let max_len = if trailing_nul { len - 1 } else { len };
        if new.len() > max_len {
            return Err(EncodeError::TooLarge);
        }
        let len_to_write = new.len().min(max_len);
        buf.extend_from_slice(&new[..len_to_write]);

        // Always pad to alignment, ensuring trailing_nul if needed
        let align_to = alignment - 1;
        let min_total = if trailing_nul {
            len_to_write + 1
        } else {
            len_to_write
        };
        let round_to = (min_total + align_to) & !align_to;
        let round_to = round_to.min(len);

        if round_to > len_to_write {
            buf.put_bytes(0, round_to - len_to_write);
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
        input
            .encode_codepage_with_alignment(&mut buf, 128, 4, false)
            .unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', b'b', b'c', 0]);
    }

    #[test]
    fn test_string_codepage_bytes_aligned_abcd() {
        let input = String::from("abcd");
        let mut buf = BytesMut::new();
        input
            .encode_codepage_with_alignment(&mut buf, 128, 4, false)
            .unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', b'b', b'c', b'd']);
    }

    #[test]
    fn test_string_codepage_bytes_trialing_nul() {
        let input = String::from("a");
        let mut buf = BytesMut::new();
        input
            .encode_codepage_with_alignment(&mut buf, 2, 2, true)
            .unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', 0]);
    }

    #[test]
    fn test_string_ascii_bytes_trialing_nul() {
        let input = String::from("ab");
        let mut buf = BytesMut::new();
        input.encode_ascii(&mut buf, 4, true).unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', b'b', 0, 0]);
    }

    #[test]
    fn test_string_ascii_too_long() {
        let input = String::from("abcde");
        let mut buf = BytesMut::new();
        let res = input.encode_ascii(&mut buf, 4, true);
        assert!(res.is_err());
    }

    #[test]
    fn test_string_codepage_too_long() {
        let input = String::from("abcde");
        let mut buf = BytesMut::new();
        let res = input.encode_codepage(&mut buf, 4, true);
        assert!(res.is_err());
    }
}
