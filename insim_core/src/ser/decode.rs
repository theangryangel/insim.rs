use bytes::{Buf, BytesMut};
use std::{error::Error, fmt};

use super::Limit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodableError {
    UnmatchedDiscrimnant(String),
    NotEnoughBytes(String),
    NeedsLimit(String),
    MissingMagic(String),
    UnexpectedLimit(String),
}

impl Error for DecodableError {}

impl fmt::Display for DecodableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodableError::UnmatchedDiscrimnant(i) => write!(f, "Unmatched Discriminant! {}", i),
            DecodableError::NotEnoughBytes(i) => write!(f, "Not enough bytes! {}", i),
            DecodableError::NeedsLimit(i) => write!(f, "Expected byte or count limit! {}", i),
            DecodableError::MissingMagic(i) => write!(f, "Missing magic: {}", i),
            DecodableError::UnexpectedLimit(limit) => write!(f, "Unexpected limit: {}", limit),
        }
    }
}

pub trait Decodable {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError>
    where
        Self: Default;
}

// bool

impl Decodable for bool {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        if limit.is_some() {
            return Err(DecodableError::UnexpectedLimit(format!("bool does not support a decode limit: {:?}", limit)))
        }

        Ok(buf.get_u8() != 0)
    }
}

// u8

impl Decodable for u8 {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        if limit.is_some() {
            return Err(DecodableError::UnexpectedLimit(format!("u8 does not support a decode limit: {:?}", limit)))
        }
        Ok(buf.get_u8())
    }
}

// u16

impl Decodable for u16 {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        if limit.is_some() {
            return Err(DecodableError::UnexpectedLimit(format!("u16 does not support a decode limit: {:?}", limit)))
        }
        Ok(buf.get_u16_le())
    }
}

// u32

impl Decodable for u32 {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        if limit.is_some() {
            return Err(DecodableError::UnexpectedLimit(format!("u32 does not support a decode limit: {:?}", limit)))
        }
        Ok(buf.get_u32_le())
    }
}

// i8

impl Decodable for i8 {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        if limit.is_some() {
            return Err(DecodableError::UnexpectedLimit(format!("i8 does not support a decode limit: {:?}", limit)))
        }
        Ok(buf.get_i8())
    }
}

// i16

impl Decodable for i16 {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        if limit.is_some() {
            return Err(DecodableError::UnexpectedLimit(format!("i16 does not support a decode limit: {:?}", limit)))
        }
        Ok(buf.get_i16_le())
    }
}

// i32

impl Decodable for i32 {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        if limit.is_some() {
            return Err(DecodableError::UnexpectedLimit(format!("i32 does not support a decode limit: {:?}", limit)))
        }
        Ok(buf.get_i32_le())
    }
}

// f32

impl Decodable for f32 {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        if limit.is_some() {
            return Err(DecodableError::UnexpectedLimit(format!("f32 does not support a decode limit: {:?}", limit)))
        }
        Ok(buf.get_f32_le())
    }
}

// Vec<T>
// We don't implement for a generic Iterator because most iteratable things
// dont really make a huge amount of sense to insim. i.e. HashMap, etc.

impl<T> Decodable for Vec<T>
where
    T: Decodable + Default + std::fmt::Debug,
{
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {

        let size = match limit {
            Some(Limit::Count(i)) => { i },

            Some(Limit::Bytes(i)) => {
                i / core::mem::size_of::<T>() 
            },

            _ => {
                return Err(DecodableError::NeedsLimit("no count provided".to_string()));
            }
        };

        let mut data = Self::default();

        for _ in 0..size {
            data.push(T::decode(buf, None)?);
        }

        Ok(data)
    }
}

// (T1, T2 ..)

impl<T1, T2> Decodable for (T1, T2)
where
    T1: Decodable + Default + std::fmt::Debug,
    T2: Decodable + Default + std::fmt::Debug,
{
    fn decode(buf: &mut BytesMut, _limit: Option<Limit>) -> Result<Self, DecodableError> {
        let t1 = T1::decode(buf, None)?;
        let t2 = T2::decode(buf, None)?;

        Ok((t1, t2))
    }
}

impl<T1, T2, T3> Decodable for (T1, T2, T3)
where
    T1: Decodable + Default + std::fmt::Debug,
    T2: Decodable + Default + std::fmt::Debug,
    T3: Decodable + Default + std::fmt::Debug,
{
    fn decode(buf: &mut BytesMut, _limit: Option<Limit>) -> Result<Self, DecodableError> {
        let t1 = T1::decode(buf, None)?;
        let t2 = T2::decode(buf, None)?;
        let t3 = T3::decode(buf, None)?;

        Ok((t1, t2, t3))
    }
}

impl<T1, T2, T3, T4> Decodable for (T1, T2, T3, T4)
where
    T1: Decodable + Default + std::fmt::Debug,
    T2: Decodable + Default + std::fmt::Debug,
    T3: Decodable + Default + std::fmt::Debug,
    T4: Decodable + Default + std::fmt::Debug,
{
    fn decode(buf: &mut BytesMut, _limit: Option<Limit>) -> Result<Self, DecodableError> {
        let t1 = T1::decode(buf, None)?;
        let t2 = T2::decode(buf, None)?;
        let t3 = T3::decode(buf, None)?;
        let t4 = T4::decode(buf, None)?;

        Ok((t1, t2, t3, t4))
    }
}
