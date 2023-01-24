pub trait Pointable: Copy + Clone + Default {}

impl Pointable for i32 {}
impl Pointable for f32 {}
impl Pointable for u16 {}

#[derive(Default, Clone, Copy, Debug)]
pub struct Point<T>
where
    T: Pointable,
{
    pub x: T,
    pub y: T,
    pub z: T,
}

impl Point<i32> {
    pub fn flipped(&self) -> Self {
        Self {
            x: self.x,
            y: -self.y,
            z: self.z,
        }
    }
}

impl Point<f32> {
    pub fn flipped(&self) -> Self {
        Self {
            x: self.x,
            y: -self.y,
            z: self.z,
        }
    }
}

use crate::{Decodable, DecodableError, Encodable, EncodableError};

impl<T> Decodable for Point<T>
where
    T: Decodable + Pointable,
{
    fn decode(buf: &mut bytes::BytesMut, count: Option<usize>) -> Result<Self, DecodableError> {
        let mut data = Self::default();
        data.x = <T>::decode(buf, count)?;
        data.y = <T>::decode(buf, count)?;
        data.z = <T>::decode(buf, count)?;
        Ok(data)
    }
}

impl<T> Encodable for Point<T>
where
    T: Encodable + Pointable,
{
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), EncodableError> {
        <T>::encode(&self.x, buf)?;
        <T>::encode(&self.y, buf)?;
        <T>::encode(&self.z, buf)?;

        Ok(())
    }
}
