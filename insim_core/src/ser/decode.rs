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

// (T1, T2 ..)

impl<T1, T2> Decodable for (T1, T2)
where
    T1: Decodable + Default + std::fmt::Debug,
    T2: Decodable + Default + std::fmt::Debug,
{
    fn decode(buf: &mut BytesMut, count: Option<usize>) -> Result<Self, DecodableError> {
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
    fn decode(buf: &mut BytesMut, count: Option<usize>) -> Result<Self, DecodableError> {
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
    fn decode(buf: &mut BytesMut, count: Option<usize>) -> Result<Self, DecodableError> {
        let t1 = T1::decode(buf, None)?;
        let t2 = T2::decode(buf, None)?;
        let t3 = T3::decode(buf, None)?;
        let t4 = T4::decode(buf, None)?;

        Ok((t1, t2, t3, t4))
    }
}

// [T; N]

macro_rules! impl_decode_slice_trait {
    ($ty:ty; $($count:expr),+ $(,)?) => {

        $(
            impl Decodable for [$ty; $count] {
                fn decode(buf: &mut BytesMut, _count: Option<usize>) -> Result<Self, DecodableError> {
                    let mut slice: [$ty; $count] = Default::default();
                    for i in 0..$count {
                        slice[i] = <$ty>::decode(buf, None)?;
                    }

                    Ok(slice)
                }
            }
        )+
    };
}

impl_decode_slice_trait!(i8; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
impl_decode_slice_trait!(i16; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
impl_decode_slice_trait!(i32; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
impl_decode_slice_trait!(u8; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
impl_decode_slice_trait!(u16; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
impl_decode_slice_trait!(u32; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
impl_decode_slice_trait!(f32; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
