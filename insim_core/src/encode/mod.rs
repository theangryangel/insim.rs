//! Encode trait
mod context;
mod error;

use std::net::Ipv4Addr;

use bytes::{BufMut, Bytes, BytesMut};
pub use context::EncodeContext;
pub use error::{EncodeError, EncodeErrorKind};

/// Enable to bytes
pub trait Encode: Sized {
    /// Indicates if this is a primitive / leaf to EncodeContext
    const PRIMITIVE: bool = false;

    /// Encode into a [`EncodeContext`]. Use this directly when you need full control over
    /// the buffer or want to append to an existing one.
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), EncodeError>;

    /// Convenience shortcut: encode into a fresh [`Bytes`] buffer. For full control use
    /// [`encode`](Self::encode) with your own [`EncodeContext`] instead.
    fn to_bytes(&self) -> Result<Bytes, EncodeError> {
        let mut buf = BytesMut::new();
        let mut ctx = EncodeContext::new(&mut buf);
        self.encode(&mut ctx)?;
        Ok(buf.freeze())
    }
}

// impls

impl Encode for char {
    const PRIMITIVE: bool = true;

    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), EncodeError> {
        if self.is_ascii() {
            ctx.buf.put_u8(*self as u8);
            return Ok(());
        }

        Err(EncodeErrorKind::NotAsciiChar { found: *self }.into())
    }
}

impl Encode for bool {
    const PRIMITIVE: bool = true;

    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), EncodeError> {
        ctx.buf.put_u8(*self as u8);
        Ok(())
    }
}

impl Encode for u8 {
    const PRIMITIVE: bool = true;

    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), EncodeError> {
        ctx.buf.put_u8(*self);

        Ok(())
    }
}

impl Encode for i8 {
    const PRIMITIVE: bool = true;

    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), EncodeError> {
        ctx.buf.put_i8(*self);

        Ok(())
    }
}

impl Encode for u16 {
    const PRIMITIVE: bool = true;

    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), EncodeError> {
        ctx.buf.put_u16_le(*self);

        Ok(())
    }
}

impl Encode for i16 {
    const PRIMITIVE: bool = true;

    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), EncodeError> {
        ctx.buf.put_i16_le(*self);

        Ok(())
    }
}

impl Encode for u32 {
    const PRIMITIVE: bool = true;

    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), EncodeError> {
        ctx.buf.put_u32_le(*self);

        Ok(())
    }
}

impl Encode for i32 {
    const PRIMITIVE: bool = true;

    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), EncodeError> {
        ctx.buf.put_i32_le(*self);

        Ok(())
    }
}

impl Encode for f32 {
    const PRIMITIVE: bool = true;

    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), EncodeError> {
        ctx.buf.put_f32_le(*self);

        Ok(())
    }
}

impl<T, const N: usize> Encode for [T; N]
where
    T: Encode,
{
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), EncodeError> {
        for i in self.iter() {
            i.encode(ctx)?;
        }

        Ok(())
    }
}

impl Encode for Ipv4Addr {
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), EncodeError> {
        let repr = u32::from(*self);
        repr.encode(ctx)
    }
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_string_codepage_bytes_aligned_abc() {
        let input = String::from("abc");
        let mut buf = BytesMut::new();
        let mut ctx = EncodeContext::new(&mut buf);
        ctx.encode_codepage("test", input, 128, Some(4), false)
            .unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', b'b', b'c', 0]);
    }

    #[test]
    fn test_string_codepage_bytes_aligned_abcd() {
        let input = String::from("abcd");
        let mut buf = BytesMut::new();
        let mut ctx = EncodeContext::new(&mut buf);
        ctx.encode_codepage("test", input, 128, Some(4), false)
            .unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', b'b', b'c', b'd']);
    }

    #[test]
    fn test_string_codepage_bytes_trialing_nul() {
        let input = String::from("a");
        let mut buf = BytesMut::new();
        let mut ctx = EncodeContext::new(&mut buf);
        ctx.encode_codepage("test", input, 2, Some(2), true)
            .unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', 0]);
    }

    #[test]
    fn test_string_ascii_bytes_trialing_nul() {
        let input = String::from("ab");
        let mut buf = BytesMut::new();
        let mut ctx = EncodeContext::new(&mut buf);
        ctx.encode_ascii("test", input, 4, true).unwrap();
        let inner = buf.freeze();
        assert_eq!(inner.as_ref(), [b'a', b'b', 0, 0]);
    }

    #[test]
    fn test_string_ascii_too_long() {
        let input = String::from("abcde");
        let mut buf = BytesMut::new();
        let mut ctx = EncodeContext::new(&mut buf);
        let res = ctx.encode_ascii("test", input, 4, true);
        assert!(res.is_err());
    }

    #[test]
    fn test_string_codepage_too_long() {
        let input = String::from("abcde");
        let mut buf = BytesMut::new();
        let mut ctx = EncodeContext::new(&mut buf);
        let res = ctx.encode_codepage("test", input, 4, None, true);
        assert!(res.is_err());
    }
}
