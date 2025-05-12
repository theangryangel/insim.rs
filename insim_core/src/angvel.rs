//! Angular Velocity

use std::fmt::Display;

use crate::{Decode, Encode};

/// AngVel
#[derive(Debug, Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AngVel(i16);

impl AngVel {
    /// 16384 = 360 deg/s
    const SCALE: f32 = 360.0 / 16384.0;

    /// Creates a new angular velocity from a raw i16 value
    pub fn new(raw: i16) -> Self {
        Self(raw)
    }

    /// Returns the raw i16 value
    pub fn raw(self) -> i16 {
        self.0
    }

    /// Converts to degrees per second
    pub fn to_degrees_sec(self) -> f32 {
        self.0 as f32 * Self::SCALE
    }

    /// Converts to radians per second
    pub fn to_radians_sec(self) -> f32 {
        self.to_degrees_sec() * std::f32::consts::PI / 180.0
    }
}

impl Display for AngVel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} 16384.0 = 180 degrees/sec", self.0)
    }
}

impl Decode for AngVel {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::DecodeError> {
        let inner = i16::decode(buf)?;
        Ok(Self(inner))
    }
}

impl Encode for AngVel {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodeError> {
        self.0.encode(buf)
    }
}
