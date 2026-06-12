//! Heading / direction wire types.
//!
//! LFS encodes headings as fixed-point integers at different resolutions and, in
//! one case, a different zero-point. Rather than decode to a lossy floating-point
//! angle and convert back (which is not idempotent on the wire), each wire encoding
//! has its own newtype that stores the **raw integer** and exposes degree/radian
//! accessors on demand. Decoding then re-encoding reproduces the original bytes
//! exactly.
//!
//! All types share the same orientation convention as LFS: angles increase
//! anticlockwise viewed from above, and [`to_degrees`](HeadingU16::to_degrees)
//! always returns a value in `[0, 360)`.
//!
//! | Type | Wire | Scale | Zero |
//! |------|------|-------|------|
//! | [`HeadingU16`] | `u16` | 32768 = 180° | 0 = 0° |
//! | [`HeadingU8`] | `u8` | 128 = 180° | 0 = 0° |
//! | [`ObjectHeading`] | `u8` | 128 = 180° | 128 = 0° (offset) |

use std::fmt;

macro_rules! define_heading {
    (
        $(#[$meta:meta])*
        $name:ident, inner = $inner:ty, half_turn = $half:expr, zero_offset = $offset:expr
    ) => {
        $(#[$meta])*
        ///
        /// Stores the raw wire value; degree/radian conversions are accessors.
        /// Decoding then encoding reproduces the original value exactly.
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
        pub struct $name($inner);

        impl $name {
            /// Raw units representing a half turn (180°).
            const HALF_TURN: u32 = $half;
            /// Raw units in a full turn (360°).
            const FULL_TURN: u32 = $half * 2;
            /// Raw value representing 0°.
            const ZERO_OFFSET: u32 = $offset;

            const fn raw_for_degrees(deg: u32) -> $inner {
                (((deg * Self::HALF_TURN / 180) + Self::ZERO_OFFSET) % Self::FULL_TURN) as $inner
            }

            /// Heading of 0° (world Y / forward).
            pub const ZERO: Self = Self(Self::raw_for_degrees(0));
            /// 0° - world Y direction (forward/north).
            pub const NORTH: Self = Self(Self::raw_for_degrees(0));
            /// 270° - world X direction (right/east).
            pub const EAST: Self = Self(Self::raw_for_degrees(270));
            /// 180° - opposite of world Y (backward/south).
            pub const SOUTH: Self = Self(Self::raw_for_degrees(180));
            /// 90° - world -X direction (left/west).
            pub const WEST: Self = Self(Self::raw_for_degrees(90));

            /// Construct from the raw wire value (lossless).
            pub const fn from_raw(raw: $inner) -> Self {
                Self(raw)
            }

            /// The raw wire value (lossless).
            pub const fn to_raw(self) -> $inner {
                self.0
            }

            /// Consumes `self`, returning the raw inner value (same as [`to_raw`](Self::to_raw)).
            pub const fn into_inner(self) -> $inner {
                self.0
            }

            /// Construct from degrees. Values outside `[0, 360)` wrap around - the
            /// wire format cannot represent an un-normalised angle.
            pub fn from_degrees(value: f64) -> Self {
                let units = (value * (Self::HALF_TURN as f64 / 180.0)).round() as i64;
                let raw = (units + Self::ZERO_OFFSET as i64).rem_euclid(Self::FULL_TURN as i64);
                Self(raw as $inner)
            }

            /// The heading in degrees, in `[0, 360)`.
            pub fn to_degrees(self) -> f64 {
                let centred =
                    (self.0 as i64 - Self::ZERO_OFFSET as i64).rem_euclid(Self::FULL_TURN as i64);
                centred as f64 * 180.0 / Self::HALF_TURN as f64
            }

            /// Construct from radians (wraps, see [`from_degrees`](Self::from_degrees)).
            pub fn from_radians(value: f64) -> Self {
                Self::from_degrees(value.to_degrees())
            }

            /// The heading in radians, in `[0, 2π)`.
            pub fn to_radians(self) -> f64 {
                self.to_degrees().to_radians()
            }

            /// The opposite direction (180° rotation). Exact - a half-turn is a
            /// whole number of wire units.
            pub fn opposite(self) -> Self {
                Self(self.0.wrapping_add(Self::HALF_TURN as $inner))
            }

            /// Normalise to `[0, 360)`. Wire headings are inherently normalised
            /// (the raw integer wraps), so this returns `self`; it exists for
            /// parity with floating-point angle APIs.
            ///
            /// Note: the American spelling `normalize` is retained (rather than the
            /// British `normalise` used elsewhere in this crate) to mirror the
            /// wider Rust ecosystem - `glam`, `nalgebra`, and `std` all spell it
            /// this way, and these headings interoperate with those libraries.
            #[doc(alias = "normalise")]
            pub fn normalize(self) -> Self {
                self
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::ZERO
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:.2}", self.to_degrees())
            }
        }

        impl crate::Decode for $name {
            fn decode(ctx: &mut crate::DecodeContext) -> Result<Self, crate::DecodeError> {
                Ok(Self(ctx.decode::<$inner>("heading")?))
            }
        }

        impl crate::Encode for $name {
            fn encode(&self, ctx: &mut crate::EncodeContext) -> Result<(), crate::EncodeError> {
                ctx.encode("heading", &self.0)
            }
        }
    };
}

define_heading! {
    /// Heading as transmitted in `CompCar` (MCI) and `Cpp`: a `u16` where
    /// 32768 = 180°, zero at 0.
    HeadingU16, inner = u16, half_turn = 32768, zero_offset = 0
}

define_heading! {
    /// Heading as transmitted in `CarContact`/`ConInfo`: a `u8` where
    /// 128 = 180°, zero at 0.
    HeadingU8, inner = u8, half_turn = 128, zero_offset = 0
}

define_heading! {
    /// Heading as transmitted in layout `ObjectInfo`: a `u8` where 128 = 180°
    /// **and the zero-point is offset** so that raw 128 = 0°
    /// (`raw = (degrees + 180) * 256 / 360`). This is why it is a distinct type
    /// from [`HeadingU8`] despite the identical scale.
    ///
    /// - 128 = 0° (forward), 192 = 90°, 0 = 180°, 64 = 270°.
    ObjectHeading, inner = u8, half_turn = 128, zero_offset = 128
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decode, Encode};

    #[test]
    fn test_headingu16_raw_roundtrip_exact() {
        for raw in [0u16, 1, 16384, 32768, 49152, 65535] {
            assert_eq!(HeadingU16::from_raw(raw).to_raw(), raw);
        }
    }

    #[test]
    fn test_headingu16_degrees() {
        assert_eq!(HeadingU16::from_raw(0).to_degrees(), 0.0);
        assert_eq!(HeadingU16::from_raw(16384).to_degrees(), 90.0);
        assert_eq!(HeadingU16::from_raw(32768).to_degrees(), 180.0);
        assert_eq!(HeadingU16::from_degrees(90.0).to_raw(), 16384);
    }

    #[test]
    fn test_headingu8_degrees() {
        assert_eq!(HeadingU8::from_raw(0).to_degrees(), 0.0);
        assert_eq!(HeadingU8::from_raw(64).to_degrees(), 90.0);
        assert_eq!(HeadingU8::from_raw(128).to_degrees(), 180.0);
    }

    #[test]
    fn test_objectheading_offset() {
        // raw 128 = 0°, 192 = 90°, 0 = 180°, 64 = 270°
        assert_eq!(ObjectHeading::from_raw(128).to_degrees(), 0.0);
        assert_eq!(ObjectHeading::from_raw(192).to_degrees(), 90.0);
        assert_eq!(ObjectHeading::from_raw(0).to_degrees(), 180.0);
        assert_eq!(ObjectHeading::from_raw(64).to_degrees(), 270.0);

        // Encode side, including the 256 -> 0 wrap at 180°.
        assert_eq!(ObjectHeading::from_degrees(0.0).to_raw(), 128);
        assert_eq!(ObjectHeading::from_degrees(90.0).to_raw(), 192);
        assert_eq!(ObjectHeading::from_degrees(180.0).to_raw(), 0);
        assert_eq!(ObjectHeading::from_degrees(-90.0).to_raw(), 64);
    }

    #[test]
    fn test_raw_roundtrip_all_u8() {
        // Every u8 must survive raw round-trip for both u8 heading types.
        for raw in 0u8..=255 {
            assert_eq!(HeadingU8::from_raw(raw).to_raw(), raw);
            assert_eq!(ObjectHeading::from_raw(raw).to_raw(), raw);
        }
    }

    #[test]
    fn test_opposite_exact() {
        assert_eq!(HeadingU16::from_raw(0).opposite().to_raw(), 32768);
        assert_eq!(HeadingU16::from_raw(32768).opposite().to_raw(), 0);
        assert_eq!(HeadingU8::from_raw(10).opposite().to_raw(), 138);
        // ObjectHeading 0° (raw 128) opposite is 180° (raw 0)
        assert_eq!(ObjectHeading::from_raw(128).opposite().to_raw(), 0);
    }

    #[test]
    fn test_cardinals() {
        assert_eq!(HeadingU16::NORTH.to_degrees(), 0.0);
        assert_eq!(HeadingU16::EAST.to_degrees(), 270.0);
        assert_eq!(HeadingU16::SOUTH.to_degrees(), 180.0);
        assert_eq!(HeadingU16::WEST.to_degrees(), 90.0);
        assert_eq!(ObjectHeading::NORTH.to_raw(), 128);
        assert_eq!(ObjectHeading::SOUTH.to_raw(), 0);
    }

    #[test]
    fn test_default_is_zero_degrees() {
        assert_eq!(HeadingU16::default().to_degrees(), 0.0);
        assert_eq!(HeadingU8::default().to_degrees(), 0.0);
        // ObjectHeading default must be 0° (raw 128), not raw 0 (which is 180°).
        assert_eq!(ObjectHeading::default().to_raw(), 128);
        assert_eq!(ObjectHeading::default().to_degrees(), 0.0);
    }

    #[test]
    fn test_byte_roundtrip() {
        for raw in [0u16, 123, 40000, 65535] {
            let h = HeadingU16::from_raw(raw);
            let bytes = h.to_bytes().unwrap();
            assert_eq!(HeadingU16::decode_slice(&bytes).unwrap(), h);
        }
        for raw in [0u8, 1, 128, 255] {
            let h = ObjectHeading::from_raw(raw);
            let bytes = h.to_bytes().unwrap();
            assert_eq!(ObjectHeading::decode_slice(&bytes).unwrap(), h);
        }
    }
}
