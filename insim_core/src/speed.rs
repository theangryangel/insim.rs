//! Speed wire types.
//!
//! LFS encodes speed in several different fixed-point scales depending on the
//! packet. Rather than decode to a lossy floating-point "metres per second" and
//! convert back (which is not idempotent on the wire), each wire encoding has its
//! own newtype that stores the **raw integer** and exposes human-unit accessors on
//! demand. Decoding then re-encoding always reproduces the original bytes exactly.
//!
//! | Type | Wire | Scale |
//! |------|------|-------|
//! | [`SpeedF32`] | `f32` | native metres per second (no scaling) |
//! | [`SpeedU16`] | `u16` | 327.68 = 1 m/s (32768 = 100 m/s) |
//! | [`SpeedU8`] | `u8` | 1 = 1 m/s |
//! | [`ClosingSpeed`] | `u16` (top 4 bits reserved) | 10 = 1 m/s |

use std::fmt;

use crate::{Decode, Encode};

/// Conversion helpers shared by the speed newtypes.
const MPS_PER_KMH: f32 = 1.0 / 3.6;
const MPS_PER_MPH: f32 = 1.0 / 2.23694;

/// Closing speed reserved-bit mask. The top 4 bits of the `spclose` field are
/// reserved; only the low 12 bits carry the value.
const CLOSING_SPEED_MASK: u16 = 0x0FFF;

/// Speed transmitted as a native IEEE `f32` in metres per second (e.g. OutGauge).
///
/// Unlike the fixed-point wire types, there is no scaling here - the wire value is
/// already an `f32` in m/s, so this wrapper is lossless. It exists for the unit
/// conversions and type-safety; the stored value is metres per second.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SpeedF32(f32);

impl SpeedF32 {
    /// Zero speed.
    pub const ZERO: Self = Self(0.0);

    /// Construct from metres per second.
    pub const fn from_metres_per_sec(value: f32) -> Self {
        Self(value)
    }

    /// Consumes `self`, returning the inner value (metres per second).
    pub const fn into_inner(self) -> f32 {
        self.0
    }

    /// Speed in metres per second.
    pub const fn to_metres_per_sec(self) -> f32 {
        self.0
    }

    /// Construct from kilometres per hour.
    pub fn from_kilometres_per_hour(value: f32) -> Self {
        Self(value * MPS_PER_KMH)
    }

    /// Speed in kilometres per hour.
    pub fn to_kilometres_per_hour(self) -> f32 {
        self.0 * 3.6
    }

    /// Construct from miles per hour.
    pub fn from_miles_per_hour(value: f32) -> Self {
        Self(value * MPS_PER_MPH)
    }

    /// Speed in miles per hour.
    pub fn to_miles_per_hour(self) -> f32 {
        self.0 * 2.23694
    }

    /// Whether this is exactly zero.
    pub fn is_zero(self) -> bool {
        self.0 == 0.0
    }
}

impl fmt::Display for SpeedF32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}m/s", self.0)
    }
}

impl Decode for SpeedF32 {
    fn decode(ctx: &mut crate::DecodeContext) -> Result<Self, crate::DecodeError> {
        Ok(Self(ctx.decode::<f32>("speed")?))
    }
}

impl Encode for SpeedF32 {
    fn encode(&self, ctx: &mut crate::EncodeContext) -> Result<(), crate::EncodeError> {
        ctx.encode("speed", &self.0)
    }
}

/// Speed as transmitted in [`Mci`](crate)'s `CompCar`: a `u16` where
/// `327.68` units = 1 m/s (i.e. `32768` = 100 m/s).
///
/// Stores the raw wire value; conversions to m/s, km/h and mph are provided as
/// accessors. Decoding then encoding reproduces the original `u16` exactly.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SpeedU16(u16);

impl SpeedU16 {
    /// Zero speed.
    pub const ZERO: Self = Self(0);

    /// Raw wire units per metre/second.
    const UNITS_PER_MPS: f32 = 327.68;

    /// Construct from the raw wire value (lossless).
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    /// The raw wire value (lossless).
    pub const fn to_raw(self) -> u16 {
        self.0
    }

    /// Construct from metres per second (rounded to the nearest wire unit).
    pub fn from_metres_per_sec(value: f32) -> Self {
        Self((value * Self::UNITS_PER_MPS).round() as u16)
    }

    /// Consumes `self`, returning the raw inner value (same as [`to_raw`](Self::to_raw)).
    pub const fn into_inner(self) -> u16 {
        self.0
    }

    /// Speed in metres per second.
    pub fn to_metres_per_sec(self) -> f32 {
        self.0 as f32 / Self::UNITS_PER_MPS
    }

    /// Construct from kilometres per hour.
    pub fn from_kilometres_per_hour(value: f32) -> Self {
        Self::from_metres_per_sec(value * MPS_PER_KMH)
    }

    /// Speed in kilometres per hour.
    pub fn to_kilometres_per_hour(self) -> f32 {
        self.to_metres_per_sec() * 3.6
    }

    /// Construct from miles per hour.
    pub fn from_miles_per_hour(value: f32) -> Self {
        Self::from_metres_per_sec(value * MPS_PER_MPH)
    }

    /// Speed in miles per hour.
    pub fn to_miles_per_hour(self) -> f32 {
        self.to_metres_per_sec() * 2.23694
    }

    /// Whether this is exactly zero.
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for SpeedU16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}m/s", self.to_metres_per_sec())
    }
}

impl Decode for SpeedU16 {
    fn decode(ctx: &mut crate::DecodeContext) -> Result<Self, crate::DecodeError> {
        Ok(Self(ctx.decode::<u16>("speed")?))
    }
}

impl Encode for SpeedU16 {
    fn encode(&self, ctx: &mut crate::EncodeContext) -> Result<(), crate::EncodeError> {
        ctx.encode("speed", &self.0)
    }
}

/// Speed as transmitted in `CarContact`/`ConInfo`: a `u8` where 1 unit = 1 m/s.
///
/// Range is therefore 0..=255 m/s in whole-metre-per-second steps. Stores the raw
/// wire value; decoding then encoding reproduces the original `u8` exactly.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SpeedU8(u8);

impl SpeedU8 {
    /// Zero speed.
    pub const ZERO: Self = Self(0);

    /// Construct from the raw wire value (lossless).
    pub const fn from_raw(raw: u8) -> Self {
        Self(raw)
    }

    /// The raw wire value (lossless).
    pub const fn to_raw(self) -> u8 {
        self.0
    }

    /// Construct from metres per second (rounded and clamped to 0..=255).
    pub fn from_metres_per_sec(value: f32) -> Self {
        Self(value.round().clamp(0.0, u8::MAX as f32) as u8)
    }

    /// Consumes `self`, returning the raw inner value (same as [`to_raw`](Self::to_raw)).
    pub const fn into_inner(self) -> u8 {
        self.0
    }

    /// Speed in metres per second.
    pub fn to_metres_per_sec(self) -> f32 {
        self.0 as f32
    }

    /// Construct from kilometres per hour.
    pub fn from_kilometres_per_hour(value: f32) -> Self {
        Self::from_metres_per_sec(value * MPS_PER_KMH)
    }

    /// Speed in kilometres per hour.
    pub fn to_kilometres_per_hour(self) -> f32 {
        self.to_metres_per_sec() * 3.6
    }

    /// Construct from miles per hour.
    pub fn from_miles_per_hour(value: f32) -> Self {
        Self::from_metres_per_sec(value * MPS_PER_MPH)
    }

    /// Speed in miles per hour.
    pub fn to_miles_per_hour(self) -> f32 {
        self.to_metres_per_sec() * 2.23694
    }

    /// Whether this is exactly zero.
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for SpeedU8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}m/s", self.0)
    }
}

impl Decode for SpeedU8 {
    fn decode(ctx: &mut crate::DecodeContext) -> Result<Self, crate::DecodeError> {
        Ok(Self(ctx.decode::<u8>("speed")?))
    }
}

impl Encode for SpeedU8 {
    fn encode(&self, ctx: &mut crate::EncodeContext) -> Result<(), crate::EncodeError> {
        ctx.encode("speed", &self.0)
    }
}

/// Closing speed as transmitted in `Con`/`Obh` `spclose`: a `u16` where 10 units =
/// 1 m/s. The top 4 bits of the wire field are reserved and are masked off on
/// decode (and never written on encode).
///
/// Stores the raw (masked) wire value; decoding then encoding reproduces the
/// original 12-bit value exactly.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ClosingSpeed(u16);

impl ClosingSpeed {
    /// Zero closing speed.
    pub const ZERO: Self = Self(0);

    /// Raw wire units per metre/second.
    const UNITS_PER_MPS: f32 = 10.0;

    /// Construct from the raw wire value. The reserved top 4 bits are masked off.
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw & CLOSING_SPEED_MASK)
    }

    /// The raw (masked, 12-bit) wire value.
    pub const fn to_raw(self) -> u16 {
        self.0
    }

    /// Construct from metres per second (rounded to the nearest wire unit).
    pub fn from_metres_per_sec(value: f32) -> Self {
        Self::from_raw((value * Self::UNITS_PER_MPS).round() as u16)
    }

    /// Consumes `self`, returning the raw (masked, 12-bit) inner value (same as
    /// [`to_raw`](Self::to_raw)).
    pub const fn into_inner(self) -> u16 {
        self.0
    }

    /// Closing speed in metres per second.
    pub fn to_metres_per_sec(self) -> f32 {
        self.0 as f32 / Self::UNITS_PER_MPS
    }

    /// Construct from kilometres per hour.
    pub fn from_kilometres_per_hour(value: f32) -> Self {
        Self::from_metres_per_sec(value * MPS_PER_KMH)
    }

    /// Closing speed in kilometres per hour.
    pub fn to_kilometres_per_hour(self) -> f32 {
        self.to_metres_per_sec() * 3.6
    }

    /// Construct from miles per hour.
    pub fn from_miles_per_hour(value: f32) -> Self {
        Self::from_metres_per_sec(value * MPS_PER_MPH)
    }

    /// Closing speed in miles per hour.
    pub fn to_miles_per_hour(self) -> f32 {
        self.to_metres_per_sec() * 2.23694
    }

    /// Whether this is exactly zero.
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for ClosingSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}m/s", self.to_metres_per_sec())
    }
}

impl Decode for ClosingSpeed {
    fn decode(ctx: &mut crate::DecodeContext) -> Result<Self, crate::DecodeError> {
        Ok(Self::from_raw(ctx.decode::<u16>("spclose")?))
    }
}

impl Encode for ClosingSpeed {
    fn encode(&self, ctx: &mut crate::EncodeContext) -> Result<(), crate::EncodeError> {
        // Defensive: the stored value is already masked, but never write reserved bits.
        ctx.encode("spclose", &(self.0 & CLOSING_SPEED_MASK))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_speedu16_raw_roundtrip_exact() {
        for raw in [0u16, 1, 327, 32768, 65535] {
            assert_eq!(SpeedU16::from_raw(raw).to_raw(), raw);
        }
    }

    #[test]
    fn test_speedu16_mps() {
        // 32768 = 100 m/s
        assert_eq!(SpeedU16::from_raw(32768).to_metres_per_sec(), 100.0);
        // round-trip through m/s lands back on the nearest raw unit
        assert_eq!(SpeedU16::from_metres_per_sec(100.0).to_raw(), 32768);
    }

    #[test]
    fn test_speedu16_kmph_mph() {
        let s = SpeedU16::from_kilometres_per_hour(100.0);
        assert!((s.to_kilometres_per_hour() - 100.0).abs() < 0.05);
        let s = SpeedU16::from_miles_per_hour(60.0);
        assert!((s.to_miles_per_hour() - 60.0).abs() < 0.05);
    }

    #[test]
    fn test_speedu8_raw_roundtrip_exact() {
        for raw in [0u8, 1, 100, 255] {
            assert_eq!(SpeedU8::from_raw(raw).to_raw(), raw);
        }
    }

    #[test]
    fn test_speedu8_clamps() {
        assert_eq!(SpeedU8::from_metres_per_sec(300.0).to_raw(), 255);
        assert_eq!(SpeedU8::from_metres_per_sec(-5.0).to_raw(), 0);
        assert_eq!(SpeedU8::from_metres_per_sec(42.0).to_raw(), 42);
    }

    #[test]
    fn test_closing_speed_strips_reserved_bits() {
        // 61441 = 0xF001 -> 0x0001
        assert_eq!(ClosingSpeed::from_raw(61441).to_raw(), 1);
        // 63495 = 0xF807 -> 0x0807 = 2055
        assert_eq!(ClosingSpeed::from_raw(63495).to_raw(), 2055);
    }

    #[test]
    fn test_closing_speed_mps() {
        // 10 units = 1 m/s
        assert_eq!(ClosingSpeed::from_raw(23).to_metres_per_sec(), 2.3);
        assert_eq!(ClosingSpeed::from_metres_per_sec(2.3).to_raw(), 23);
    }

    #[test]
    fn test_is_zero_and_default() {
        assert!(SpeedU16::ZERO.is_zero());
        assert!(SpeedU8::ZERO.is_zero());
        assert!(ClosingSpeed::ZERO.is_zero());
        assert_eq!(SpeedU16::default(), SpeedU16::ZERO);
        assert_eq!(SpeedU8::default(), SpeedU8::ZERO);
        assert_eq!(ClosingSpeed::default(), ClosingSpeed::ZERO);
    }
}
