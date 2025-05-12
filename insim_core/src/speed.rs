//! Utilities for speed
use std::{fmt, marker::PhantomData};

use num_traits::Num;

use crate::{Decode, Encode};

/// SpeedKind
pub trait SpeedKind: Copy {
    /// Raw/Inner type
    type Inner: fmt::Display + Copy + Num + Decode + Encode + Default;

    /// Display name, used in Display implementation
    fn name() -> &'static str;

    /// Convert from meters per sec
    fn from_meters_per_sec(value: f32) -> Self::Inner;
    /// Convert into meters per sec
    fn to_meters_per_sec(value: Self::Inner) -> f32;
}

/// Representation of Speed. The intention is to keep things as close to raw as possible, whilst
/// allowing to convert to and from human useful terms like mph, kmph, mps, etc. whilst maintaining
/// type safety.
/// There is no intention to create a complex uom-style system.
/// Previous generations of Speed were more akin to Duration from the standard library, but this
/// exposed a design conflict.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Speed<T: SpeedKind> {
    inner: T::Inner,
    _marker: PhantomData<T>,
}

impl<T> Speed<T>
where
    T: SpeedKind,
{
    /// New
    pub fn new(value: T::Inner) -> Self {
        Self {
            inner: value,
            _marker: PhantomData,
        }
    }

    /// Consumes Speed, returning the raw wrapped value.
    pub fn into_inner(self) -> T::Inner {
        self.inner
    }

    /// Convert from meters per sec
    pub fn from_meters_per_sec(value: f32) -> Self {
        Self::new(T::from_meters_per_sec(value))
    }
    /// Convert into meters per sec
    pub fn to_meters_per_sec(&self) -> f32 {
        T::to_meters_per_sec(self.inner)
    }

    /// Convert from kph
    pub fn from_kilometers_per_hour(value: f32) -> Self {
        Self::new(T::from_meters_per_sec(value / 3.6))
    }
    /// Convert into kph
    pub fn to_kilometers_per_hour(&self) -> f32 {
        T::to_meters_per_sec(self.inner) * 3.6
    }

    /// Convert from mph
    pub fn from_miles_per_hour(value: f32) -> Self {
        Self::new(T::from_meters_per_sec(value / 2.23694))
    }
    /// Convert into mph
    pub fn to_miles_per_hour(&self) -> f32 {
        T::to_meters_per_sec(self.inner) * 2.23694
    }
}

impl<T: SpeedKind> fmt::Display for Speed<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.inner, T::name())
    }
}

impl<T: SpeedKind> Default for Speed<T> {
    fn default() -> Self {
        Self::new(T::Inner::default())
    }
}

impl<T> Encode for Speed<T>
where
    T: SpeedKind,
{
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodeError> {
        self.inner.encode(buf)
    }
}

impl<T> Decode for Speed<T>
where
    T: SpeedKind,
{
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::DecodeError> {
        let inner = T::Inner::decode(buf)?;
        Ok(Self::new(inner))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    struct SpeedKind32768;
    impl SpeedKind for SpeedKind32768 {
        type Inner = u16;

        fn name() -> &'static str {
            "SpeedKind32768"
        }

        fn from_meters_per_sec(value: f32) -> Self::Inner {
            (value * 327.68) as Self::Inner
        }

        fn to_meters_per_sec(value: Self::Inner) -> f32 {
            (value as f32) / 327.68
        }
    }

    #[derive(Copy, Clone, Debug)]
    struct SpeedKind1;
    impl SpeedKind for SpeedKind1 {
        type Inner = u8;

        fn name() -> &'static str {
            "SpeedKind1"
        }

        fn from_meters_per_sec(value: f32) -> Self::Inner {
            value as Self::Inner
        }

        fn to_meters_per_sec(value: Self::Inner) -> f32 {
            value as f32
        }
    }

    #[test]
    fn test_speedkind_32768_meters_per_sec() {
        assert_eq!(
            Speed::<SpeedKind32768>::new(32768).to_meters_per_sec(),
            100.0
        );
        assert_eq!(
            Speed::<SpeedKind32768>::from_meters_per_sec(100.0).into_inner(),
            32768
        );
    }

    #[test]
    fn test_speedkind_32768_kmph() {
        assert_eq!(
            Speed::<SpeedKind32768>::from_kilometers_per_hour(100.0).into_inner(),
            9102
        );
    }

    #[test]
    fn test_speedkind_32768_mph_kmph() {
        assert_eq!(
            Speed::<SpeedKind32768>::from_kilometers_per_hour(100.0).to_miles_per_hour(),
            62.135704
        );
    }

    #[test]
    fn test_speedkind_1_meters_per_sec() {
        assert_eq!(Speed::<SpeedKind1>::new(1).to_meters_per_sec(), 1.0);
        assert_eq!(
            Speed::<SpeedKind1>::from_meters_per_sec(1.0).into_inner(),
            1
        );
    }
}
