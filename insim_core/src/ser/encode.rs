use bytes::{BufMut, BytesMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodableError {
    TooLarge(String),
}

pub trait Encodable {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError>
    where
        Self: Sized;
}

// bool

impl Encodable for bool {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError> {
        buf.put_u8(*self as u8);
        Ok(())
    }
}

// u8

impl Encodable for u8 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError> {
        buf.put_u8(*self);
        Ok(())
    }
}

// u16

impl Encodable for u16 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError> {
        buf.put_u16_le(*self);
        Ok(())
    }
}

// u32

impl Encodable for u32 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError> {
        buf.put_u32_le(*self);
        Ok(())
    }
}

// i8

impl Encodable for i8 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError> {
        buf.put_i8(*self);
        Ok(())
    }
}

// i16

impl Encodable for i16 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError> {
        buf.put_i16_le(*self);
        Ok(())
    }
}

// i32

impl Encodable for i32 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError> {
        buf.put_i32_le(*self);
        Ok(())
    }
}

// f32

impl Encodable for f32 {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError> {
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
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError> {
        for i in self {
            i.encode(buf)?;
        }
        Ok(())
    }
}

// (T1, T2, ..)
//
impl<T1, T2> Encodable for (T1, T2)
where
    T1: Encodable,
    T2: Encodable,
{
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError> {
        self.0.encode(buf)?;
        self.1.encode(buf)?;
        Ok(())
    }
}

impl<T1, T2, T3> Encodable for (T1, T2, T3)
where
    T1: Encodable,
    T2: Encodable,
    T3: Encodable,
{
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError> {
        self.0.encode(buf)?;
        self.1.encode(buf)?;
        self.2.encode(buf)?;
        Ok(())
    }
}

impl<T1, T2, T3, T4> Encodable for (T1, T2, T3, T4)
where
    T1: Encodable,
    T2: Encodable,
    T3: Encodable,
    T4: Encodable,
{
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError> {
        self.0.encode(buf)?;
        self.1.encode(buf)?;
        self.2.encode(buf)?;
        self.3.encode(buf)?;
        Ok(())
    }
}

// [T; N]

impl<T, const N: usize> Encodable for [T; N]
where
    T: Encodable,
{
    fn encode(&self, buf: &mut BytesMut) -> Result<(), EncodableError> {
        for i in self {
            i.encode(buf)?;
        }
        Ok(())
    }
}
