//! Utilities for working with various strings from Insim.

use bytes::{BufMut, Bytes, BytesMut};

use crate::Error;

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

/// Read from bytes
pub trait Ascii: Sized {
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

impl Ascii for String {
    fn from_ascii_bytes(buf: &mut Bytes, len: usize) -> Result<Self, Error> {
        let new = buf.split_to(len);
        let bytes = strip_trailing_nul(&new);
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
pub trait Codepage {
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

impl<T> Codepage for T
where
    T: AsRef<str>,
{
    fn from_codepage_bytes(buf: &mut Bytes, len: usize) -> Result<String, Error> {
        let new = buf.split_to(buf.len().min(len));
        let new = codepages::to_lossy_string(strip_trailing_nul(&new));
        Ok(new.to_string())
    }

    fn to_codepage_bytes(
        &self,
        buf: &mut BytesMut,
        len: usize,
        trailing_nul: bool,
    ) -> Result<(), Error> {
        let new = codepages::to_lossy_bytes(self.as_ref());
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
        let new = codepages::to_lossy_bytes(self.as_ref());
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
