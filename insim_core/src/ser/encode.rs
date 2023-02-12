use bytes::{BufMut, BytesMut};
use std::{error::Error, fmt};

use super::Limit;
use crate::string::codepages;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodableError {
    TooLarge(String),
    UnexpectedLimit(String),
}

impl Error for EncodableError {}

impl fmt::Display for EncodableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodableError::TooLarge(i) => write!(f, "Too large! {i}"),
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
            return Err(EncodableError::TooLarge(format!(
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

// String

impl Encodable for String {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError> {
        let data = codepages::to_lossy_bytes(self);
        data.encode(buf, limit)?;
        Ok(())
    }
}
