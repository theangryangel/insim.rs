use bytes::{BufMut, BytesMut};
use std::{error::Error, fmt, net::Ipv4Addr, time::Duration};

use super::Limit;
use crate::string::codepages;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodableError {
    WrongSize(String),
    UnexpectedLimit(String),
}

impl Error for EncodableError {}

impl fmt::Display for EncodableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodableError::WrongSize(i) => write!(f, "Wrong Size! {i}"),
            EncodableError::UnexpectedLimit(i) => write!(f, "Unexpected limit! {i}"),
        }
    }
}

pub trait Encodable {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError>
    where
        Self: Sized;
}

// bool

impl Encodable for bool {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError> {
        (*self as u8).encode(buf, limit)
    }
}

// u8

impl Encodable for u8 {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError> {
        if let Some(limit) = limit {
            return Err(EncodableError::UnexpectedLimit(format!(
                "u8 does not support a limit: {limit:?}"
            )));
        }

        buf.put_u8(*self);
        Ok(())
    }
}

// u16

impl Encodable for u16 {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError> {
        if let Some(limit) = limit {
            return Err(EncodableError::UnexpectedLimit(format!(
                "u16 does not support a limit: {limit:?}"
            )));
        }

        buf.put_u16_le(*self);
        Ok(())
    }
}

// u32

impl Encodable for u32 {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError> {
        if let Some(limit) = limit {
            return Err(EncodableError::UnexpectedLimit(format!(
                "u32 does not support a limit: {limit:?}"
            )));
        }

        buf.put_u32_le(*self);
        Ok(())
    }
}

// i8

impl Encodable for i8 {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError> {
        if let Some(limit) = limit {
            return Err(EncodableError::UnexpectedLimit(format!(
                "i8 does not support a limit: {limit:?}"
            )));
        }

        buf.put_i8(*self);
        Ok(())
    }
}

// i16

impl Encodable for i16 {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError> {
        if let Some(limit) = limit {
            return Err(EncodableError::UnexpectedLimit(format!(
                "i16 does not support a limit: {limit:?}"
            )));
        }

        buf.put_i16_le(*self);
        Ok(())
    }
}

// i32

impl Encodable for i32 {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError> {
        if let Some(limit) = limit {
            return Err(EncodableError::UnexpectedLimit(format!(
                "i32 does not support a limit: {limit:?}"
            )));
        }

        buf.put_i32_le(*self);
        Ok(())
    }
}

// f32

impl Encodable for f32 {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError> {
        if let Some(limit) = limit {
            return Err(EncodableError::UnexpectedLimit(format!(
                "f32 does not support a limit: {limit:?}"
            )));
        }

        buf.put_f32_le(*self);
        Ok(())
    }
}

// Vec<T>
// We don't implement for a generic Iterator because most iteratable things
// dont really make a huge amount of sense to insim. i.e. HashMap, etc.

impl<T> Encodable for Vec<T>
where
    T: Encodable,
{
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError> {
        let limit_as_iterations = match limit {
            Some(Limit::Count(i)) => i,
            Some(Limit::Bytes(i)) => i / core::mem::size_of::<T>(),
            None => self.len(),
        };

        if limit_as_iterations < self.len() {
            return Err(EncodableError::WrongSize(format!(
                "Limit of {limit_as_iterations}, with {} items in vec",
                self.len()
            )));
        }

        for i in self {
            i.encode(buf, None)?;
        }
        Ok(())
    }
}

// Slices

impl<T, const N: usize> Encodable for [T; N]
where
    T: Encodable,
{
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError>
    where
        Self: Sized,
    {
        if limit.is_some() {
            return Err(EncodableError::UnexpectedLimit(format!(
                "Slices do not support a limit! {limit:?}"
            )));
        }

        for i in self {
            i.encode(buf, None)?;
        }

        Ok(())
    }
}

// String

impl Encodable for String {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError> {
        let mut data = codepages::to_lossy_bytes(self);

        match limit {
            Some(Limit::Count(_)) => {
                return Err(EncodableError::UnexpectedLimit(format!(
                    "String does not support a count limit! {limit:?}"
                )));
            }
            Some(Limit::Bytes(size)) => {
                if data.len() > size {
                    return Err(EncodableError::WrongSize(format!(
                        "Could not fit {} bytes into field with {size} bytes limit",
                        data.len()
                    )));
                }

                if data.len() < size {
                    // zero pad
                    data.put_bytes(0, size - data.len());
                }
            }
            _ => {}
        }

        data.encode(buf, limit)?;
        Ok(())
    }
}

// Duration

impl Encodable for Duration {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError>
    where
        Self: Sized,
    {
        let millis = self.as_millis() as u32;
        millis.encode(buf, limit)
    }
}

// Ipv4Addr

impl Encodable for Ipv4Addr {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError>
    where
        Self: Sized,
    {
        Into::<u32>::into(*self).encode(buf, limit)
    }
}
