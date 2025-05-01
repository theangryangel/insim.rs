//! Decode trait

use std::{array::from_fn, net::Ipv4Addr};

use bytes::{Buf, Bytes};

use crate::Error;

/// Decode from bytes
pub trait Decode: Sized {
    /// Read
    fn decode(buf: &mut Bytes) -> Result<Self, Error>;
}

impl Decode for char {
    fn decode(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u8() as char)
    }
}

impl Decode for bool {
    fn decode(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u8() > 0)
    }
}

impl Decode for u8 {
    fn decode(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u8())
    }
}

impl Decode for u16 {
    fn decode(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u16_le())
    }
}

impl Decode for i16 {
    fn decode(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_i16_le())
    }
}

impl Decode for u32 {
    fn decode(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_u32_le())
    }
}

impl Decode for i32 {
    fn decode(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_i32_le())
    }
}

impl Decode for f32 {
    fn decode(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(buf.get_f32_le())
    }
}

impl<T, const N: usize> Decode for [T; N]
where
    T: Decode,
{
    fn decode(buf: &mut Bytes) -> Result<Self, Error> {
        let val = from_fn(|_| T::decode(buf).unwrap());

        Ok(val)
    }
}

impl Decode for Ipv4Addr {
    fn decode(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(Ipv4Addr::from(u32::decode(buf)?))
    }
}
