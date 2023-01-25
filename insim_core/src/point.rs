pub trait Pointable: Copy + Clone + Default {}

impl Pointable for i32 {}
impl Pointable for f32 {}
impl Pointable for u16 {}

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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

use crate::{Decodable, DecodableError, Encodable, EncodableError, ser::Limit};

impl<T> Decodable for Point<T>
where
    T: Decodable + Pointable,
{
    fn decode(buf: &mut bytes::BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        if limit.is_some() {
            return Err(DecodableError::UnexpectedLimit(format!("limit is not supported on Point<T>: {:?}", limit)))
        }
        let mut data = Self::default();
        data.x = <T>::decode(buf, None)?;
        data.y = <T>::decode(buf, None)?;
        data.z = <T>::decode(buf, None)?;
        Ok(data)
    }
}

impl<T> Encodable for Point<T>
where
    T: Encodable + Pointable,
{
    fn encode(&self, buf: &mut bytes::BytesMut, limit: Option<Limit>) -> Result<(), EncodableError> {
        if limit.is_some() {
            return Err(EncodableError::UnexpectedLimit(format!("limit is not supported on Point<T>: {:?}", limit)))
        }
        <T>::encode(&self.x, buf, None)?;
        <T>::encode(&self.y, buf, None)?;
        <T>::encode(&self.z, buf, None)?;

        Ok(())
    }
}
