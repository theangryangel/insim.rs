//! Decode trait

mod context;
mod error;

use std::net::Ipv4Addr;

use arrayvec::ArrayVec;
use bytes::Buf;
pub use context::DecodeContext;
pub use error::{DecodeError, DecodeErrorKind};

/// Decode from bytes
pub trait Decode: Sized {
    /// Indicates if this is a primitive / leaf to DecodeContext
    const PRIMITIVE: bool = false;

    /// Read
    fn decode(ctx: &mut DecodeContext) -> Result<Self, DecodeError>;
}

// impls

impl Decode for char {
    const PRIMITIVE: bool = true;

    fn decode(ctx: &mut DecodeContext) -> Result<Self, DecodeError> {
        Ok(ctx.buf.get_u8() as char)
    }
}

impl Decode for bool {
    const PRIMITIVE: bool = true;

    fn decode(ctx: &mut DecodeContext) -> Result<Self, DecodeError> {
        Ok(ctx.buf.get_u8() > 0)
    }
}

impl Decode for u8 {
    const PRIMITIVE: bool = true;

    fn decode(ctx: &mut DecodeContext) -> Result<Self, DecodeError> {
        Ok(ctx.buf.get_u8())
    }
}

impl Decode for u16 {
    const PRIMITIVE: bool = true;

    fn decode(ctx: &mut DecodeContext) -> Result<Self, DecodeError> {
        Ok(ctx.buf.get_u16_le())
    }
}

impl Decode for i16 {
    const PRIMITIVE: bool = true;

    fn decode(ctx: &mut DecodeContext) -> Result<Self, DecodeError> {
        Ok(ctx.buf.get_i16_le())
    }
}

impl Decode for u32 {
    const PRIMITIVE: bool = true;

    fn decode(ctx: &mut DecodeContext) -> Result<Self, DecodeError> {
        Ok(ctx.buf.get_u32_le())
    }
}

impl Decode for i32 {
    const PRIMITIVE: bool = true;

    fn decode(ctx: &mut DecodeContext) -> Result<Self, DecodeError> {
        Ok(ctx.buf.get_i32_le())
    }
}

impl Decode for f32 {
    const PRIMITIVE: bool = true;

    fn decode(ctx: &mut DecodeContext) -> Result<Self, DecodeError> {
        Ok(ctx.buf.get_f32_le())
    }
}

impl<T, const N: usize> Decode for [T; N]
where
    T: Decode,
{
    fn decode(ctx: &mut DecodeContext) -> Result<Self, DecodeError> {
        // TODO: waiting for `std::array::try_from_fn` to stablise.
        // For now we'll use ArrayVec to reduce the allocation count that the previous
        // implementation using Vec had.
        // This is a choice based on reducing handling higher frequency packets, such as MCI.

        let mut vec = ArrayVec::<T, N>::new();
        for _ in 0..N {
            vec.push(T::decode(ctx)?);
        }
        // expect is safe since we pushed exactly N
        Ok(vec
            .into_inner()
            .ok()
            .expect("size must match N because we looped N times"))
    }
}

impl Decode for Ipv4Addr {
    fn decode(ctx: &mut DecodeContext) -> Result<Self, DecodeError> {
        Ok(Ipv4Addr::from(u32::decode(ctx)?))
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
        fn decode(ctx: &mut DecodeContext) -> Result<Self, DecodeError> {
            match u8::decode(ctx)? {
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
        fn decode(ctx: &mut DecodeContext) -> Result<Self, DecodeError> {
            let status = Status::decode(ctx).map_err(|e| e.nested().context("Outer::status"))?;
            Ok(Self { status })
        }
    }

    #[test]
    fn test_nested_error_chain() {
        // Invalid status byte (99 doesn't match any variant)
        let mut buf = Bytes::from_static(&[99]);
        let mut ctx = DecodeContext::new(&mut buf);
        let result = Outer::decode(&mut ctx);

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Outer::status"));
        assert!(err_msg.contains("no variant match"));
    }
}
