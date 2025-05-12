//! Direction
use std::{fmt, marker::PhantomData};

use num_traits::Num;

use crate::{Decode, Encode};

/// DirectionKind
pub trait DirectionKind: Copy {
    /// Raw/Inner type
    type Inner: fmt::Display + Copy + Num + Decode + Encode + Default;

    /// Display name, used in Display implementation
    fn name() -> &'static str;

    /// Convert from radians
    fn from_radians(value: f32) -> Self::Inner;
    /// Convert into radians
    fn to_radians(value: Self::Inner) -> f32;
}

/// Representation of Direction. The intention is to keep things as close to raw as possible, whilst
/// allowing to convert to and from human useful terms like mph, kmph, mps, etc. whilst maintaining
/// type safety.
/// There is no intention to create a complex uom-style system.
/// Previous generations of Direction were more akin to Duration from the standard library, but this
/// exposed a design conflict.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Direction<T: DirectionKind> {
    inner: T::Inner,
    _marker: PhantomData<T>,
}

impl<T> Direction<T>
where
    T: DirectionKind,
{
    /// New
    pub fn new(value: T::Inner) -> Self {
        Self {
            inner: value,
            _marker: PhantomData,
        }
    }

    /// Consumes Direction, returning the raw wrapped value.
    pub fn into_inner(self) -> T::Inner {
        self.inner
    }

    /// Convert from degrees
    pub fn from_degrees(value: f32) -> Self {
        Self::new(T::from_radians(value.to_radians()))
    }
    /// Convert into degrees
    pub fn to_degrees(&self) -> f32 {
        T::to_radians(self.inner).to_degrees()
    }

    /// Convert from radians
    pub fn from_radians(value: f32) -> Self {
        Self::new(T::from_radians(value))
    }
    /// Convert into radians
    pub fn to_radians(&self) -> f32 {
        T::to_radians(self.inner)
    }
}

impl<T: DirectionKind> fmt::Display for Direction<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.inner, T::name())
    }
}

impl<T: DirectionKind> Default for Direction<T> {
    fn default() -> Self {
        Self::new(T::Inner::default())
    }
}

impl<T> Encode for Direction<T>
where
    T: DirectionKind,
{
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodeError> {
        self.inner.encode(buf)
    }
}

impl<T> Decode for Direction<T>
where
    T: DirectionKind,
{
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::DecodeError> {
        let inner = T::Inner::decode(buf)?;
        Ok(Self::new(inner))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Copy, Clone, Debug, Default)]
    pub struct Direction32768;

    impl DirectionKind for Direction32768 {
        type Inner = u16;

        fn name() -> &'static str {
            "32768 = 180 deg"
        }

        fn from_radians(value: f32) -> Self::Inner {
            ((value * 32768.0 / std::f32::consts::PI)
                .round()
                .clamp(0.0, 65535.0)) as u16
        }

        fn to_radians(value: Self::Inner) -> f32 {
            (value as f32) * std::f32::consts::PI / 32768.0
        }
    }

    #[test]
    fn test_direction_game_units() {
        assert_eq!(Direction::<Direction32768>::new(32768).to_degrees(), 180.0);
        assert_eq!(
            Direction::<Direction32768>::from_degrees(180.0).into_inner(),
            32768
        );
    }
}
