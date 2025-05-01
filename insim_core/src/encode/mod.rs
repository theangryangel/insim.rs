//! Encode trait

use std::net::Ipv4Addr;

use bytes::{BufMut, BytesMut};

use crate::Error;

/// Enable to bytes
pub trait Encode: Sized {
    /// Write
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Error>;
}

impl Encode for char {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Error> {
        if self.is_ascii() {
            buf.put_u8(*self as u8);
            return Ok(());
        }

        Err(Error::NotAsciiChar { found: *self })
    }
}

impl Encode for bool {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_u8(*self as u8);
        Ok(())
    }
}

impl Encode for u8 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_u8(*self);

        Ok(())
    }
}

impl Encode for u16 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_u16_le(*self);

        Ok(())
    }
}

impl Encode for i16 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_i16_le(*self);

        Ok(())
    }
}

impl Encode for u32 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_u32_le(*self);

        Ok(())
    }
}

impl Encode for i32 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_i32_le(*self);

        Ok(())
    }
}

impl Encode for f32 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Error> {
        buf.put_f32_le(*self);

        Ok(())
    }
}

impl<T, const N: usize> Encode for [T; N]
where
    T: Encode,
{
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Error> {
        for i in self.iter() {
            i.encode(buf)?;
        }

        Ok(())
    }
}

impl Encode for Ipv4Addr {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Error> {
        let repr = u32::from(*self);
        repr.encode(buf)
    }
}
