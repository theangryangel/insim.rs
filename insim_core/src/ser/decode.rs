use bytes::{Buf, BytesMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodableError {
    UnmatchedDiscrimnant(String),
    NotEnoughBytes(String),
    NeedsCount(String),
}

pub trait Decodable {
    fn decode(buf: &mut BytesMut, count: Option<usize>) -> Result<Self, DecodableError>
    where
        Self: Default;
}

// bool

impl Decodable for bool {
    fn decode(buf: &mut BytesMut, _count: Option<usize>) -> Result<Self, DecodableError> {
        Ok(buf.get_u8() != 0)
    }
}

// u8

impl Decodable for u8 {
    fn decode(buf: &mut BytesMut, _count: Option<usize>) -> Result<Self, DecodableError> {
        Ok(buf.get_u8())
    }
}

// u16

impl Decodable for u16 {
    fn decode(buf: &mut BytesMut, _count: Option<usize>) -> Result<Self, DecodableError> {
        Ok(buf.get_u16_le())
    }
}

// u32

impl Decodable for u32 {
    fn decode(buf: &mut BytesMut, _count: Option<usize>) -> Result<Self, DecodableError> {
        Ok(buf.get_u32_le())
    }
}

// i8

impl Decodable for i8 {
    fn decode(buf: &mut BytesMut, _count: Option<usize>) -> Result<Self, DecodableError> {
        Ok(buf.get_i8())
    }
}

// i16

impl Decodable for i16 {
    fn decode(buf: &mut BytesMut, _count: Option<usize>) -> Result<Self, DecodableError> {
        Ok(buf.get_i16_le())
    }
}

// i32

impl Decodable for i32 {
    fn decode(buf: &mut BytesMut, _count: Option<usize>) -> Result<Self, DecodableError> {
        Ok(buf.get_i32_le())
    }
}

// f32

impl Decodable for f32 {
    fn decode(buf: &mut BytesMut, _count: Option<usize>) -> Result<Self, DecodableError> {
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
    fn decode(buf: &mut BytesMut, count: Option<usize>) -> Result<Self, DecodableError> {
        if count.is_none() {
            return Err(DecodableError::NeedsCount("no count provided".to_string()));
        }

        let size = count.unwrap();

        let mut data = Self::default();

        for _ in 0..size {
            data.push(T::decode(buf, None)?);
        }

        Ok(data)
    }
}
