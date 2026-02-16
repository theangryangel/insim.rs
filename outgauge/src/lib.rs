#![doc = include_str!("../README.md")]
#![cfg_attr(test, deny(warnings, unreachable_pub))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use std::{
    fmt,
    ops::{Deref, DerefMut},
    time::Duration,
};

pub use ::insim_core as core;
use bytes::{Buf, BufMut};
use insim_core::{
    Decode, DecodeString, Encode, EncodeString, dash_lights::DashLights, gear::Gear,
    identifiers::PlayerId, speed::Speed, vehicle::Vehicle,
};

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    /// Describes the setup of a player and the various helpers that may be enabled, such as
    /// auto-clutch, etc.
    pub struct OutgaugeFlags: u16 {
        /// Shift key
        const SHIFT = 1;
        /// Control key
        const CTRL = (1 << 1);
        /// Show Turbo
        const TURBO = (1 << 13);
        /// Prefer Kilometers, if not set, prefer miles
        const KM = (1 << 14);
        /// Prefer bar, if not set, prefer PSI
        const BAR = (1 << 15);
    }
}

impl Encode for OutgaugeFlags {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.bits().encode(buf)
    }
}

impl Decode for OutgaugeFlags {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        Ok(Self::from_bits_truncate(u16::decode(buf)?))
    }
}

/// Unique Player Identifier, commonly referred to as PLID in Insim.txt
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutgaugeId(pub i32);

impl fmt::Display for OutgaugeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for OutgaugeId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OutgaugeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<i32> for OutgaugeId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl Decode for OutgaugeId {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        Ok(OutgaugeId(buf.get_i32_le()))
    }
}

impl Encode for OutgaugeId {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        buf.put_i32_le(self.0);

        Ok(())
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Outgauge packet
pub struct Outgauge {
    /// Time, useful for ordering
    pub time: Duration,
    /// Vehicle name
    pub car: Vehicle,
    /// Flags describing what and how to display
    pub flags: OutgaugeFlags,
    /// Current gear: reverse=0, neutral=1, first=2...
    pub gear: Gear,
    /// Currently viewed player
    pub plid: PlayerId,
    /// Speed in m/s
    pub speed: Speed,
    /// RPM
    pub rpm: f32,
    /// Turbo pressure
    pub turbo: f32,
    /// Engine temp in celcius
    pub engtemp: f32,
    /// Fuel percentage, from 0-1
    pub fuel: f32,
    /// Oil pressure
    pub oilpressure: f32,
    /// Oil temp
    pub oiltemp: f32,
    /// Available dashboard lights
    pub dashlights: DashLights,
    /// Iluminated dashboard lights
    pub showlights: DashLights,
    /// Throttle percentage, 0-1
    pub throttle: f32,
    /// Brake percentage, 0-1
    pub brake: f32,
    /// Clutch percentage, 0-1
    pub clutch: f32,
    /// Display text, usually fuel
    pub display1: String,
    /// Display text, usually settings
    pub display2: String,

    /// Optional identifier
    pub id: Option<OutgaugeId>,
}

impl Encode for Outgauge {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        let time = self.time.as_millis();
        (time as u32).encode(buf)?;
        self.car.encode(buf)?;
        self.flags.encode(buf)?;
        self.gear.encode(buf)?;
        self.plid.encode(buf)?;
        self.speed.to_meters_per_sec().encode(buf)?;
        self.rpm.encode(buf)?;
        self.turbo.encode(buf)?;
        self.engtemp.encode(buf)?;
        self.fuel.encode(buf)?;
        self.oilpressure.encode(buf)?;
        self.oiltemp.encode(buf)?;
        self.dashlights.encode(buf)?;
        self.showlights.encode(buf)?;
        self.throttle.encode(buf)?;
        self.clutch.encode(buf)?;
        self.brake.encode(buf)?;
        self.display1.encode_ascii(buf, 16, false)?;
        self.display2.encode_ascii(buf, 16, false)?;
        if let Some(id) = self.id {
            id.encode(buf)?;
        }
        Ok(())
    }
}

impl Decode for Outgauge {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let time = Duration::from_millis(u32::decode(buf)? as u64);
        let car = Vehicle::decode(buf)?;
        let flags = OutgaugeFlags::decode(buf)?;
        let gear = Gear::decode(buf)?;
        let plid = PlayerId::decode(buf)?;
        let speed = Speed::from_meters_per_sec(f32::decode(buf)?);
        let rpm = f32::decode(buf)?;
        let turbo = f32::decode(buf)?;
        let engtemp = f32::decode(buf)?;
        let fuel = f32::decode(buf)?;
        let oilpressure = f32::decode(buf)?;
        let oiltemp = f32::decode(buf)?;

        let dashlights = DashLights::decode(buf)?;
        let showlights = DashLights::decode(buf)?;

        let throttle = f32::decode(buf)?;
        let clutch = f32::decode(buf)?;
        let brake = f32::decode(buf)?;

        let display1 = String::decode_ascii(buf, 16)?;
        let display2 = String::decode_ascii(buf, 16)?;

        let id = if buf.has_remaining() {
            Some(OutgaugeId::decode(buf)?)
        } else {
            None
        };

        Ok(Self {
            time,
            car,
            flags,
            gear,
            plid,
            speed,
            rpm,
            turbo,
            engtemp,
            fuel,
            oilpressure,
            oiltemp,
            dashlights,
            showlights,
            throttle,
            clutch,
            brake,
            display1,
            display2,
            id,
        })
    }
}

#[cfg(test)]
mod test {
    use bytes::{BufMut, BytesMut};

    use super::*;

    const RAW: [u8; 92] = [
        152, // Time (1)
        7,   // Time (2)
        1,   // Time (3)
        0,   // Time (4)
        88,  // Car (1)
        82,  // Car (2)
        84,  // Car (3)
        0,   // Car (4)
        0,   // Flags (1)
        224, // Flags (2)
        3,   // Gear
        1,   // PLID
        232, // Speed (1)
        40,  // Speed (2)
        51,  // Speed (3)
        65,  // Speed (4)
        64,  // RPM (1)
        38,  // RPM (2)
        25,  // RPM (3)
        69,  // RPM (4)
        83,  // Turbo (1)
        255, // Turbo (2)
        127, // Turbo (3)
        191, // Turbo (4)
        0,   // EngTemp (1)
        0,   // EngTemp (2)
        0,   // EngTemp (3)
        0,   // EngTemp (4)
        79,  // Fuel (1)
        246, // Fuel (2)
        127, // Fuel (3)
        63,  // Fuel (4)
        0,   // OilPressure (1)
        0,   // OilPressure (2)
        0,   // OilPressure (3)
        0,   // OilPressure (4)
        0,   // OilTemp (1)
        0,   // OilTemp (2)
        0,   // OilTemp (3)
        0,   // OilTemp (4)
        102, // DashLights (1)
        7,   // DashLights (2)
        0,   // DashLights (3)
        0,   // DashLights (4)
        0,   // ShowLights (1)
        0,   // ShowLights (2)
        0,   // ShowLights (3)
        0,   // ShowLights (4)
        64,  // Throttle (1)
        38,  // Throttle (2)
        25,  // Throttle (3)
        60,  // Throttle (4)
        64,  // Brake (1)
        32,  // Brake (2)
        32,  // Brake (3)
        55,  // Brake (4)
        66,  // Clutch (1)
        33,  // Clutch (2)
        34,  // Clutch (3)
        62,  // Clutch (4)
        70,  // Display1[16]
        117, 101, 108, 32, 57, 57, 46, 57, 37, 32, 32, 32, 0, 0, 0, 66, // Display2[16]
        114, 97, 107, 101, 32, 66, 97, 108, 32, 70, 114, 32, 55, 53, 37,
    ];

    #[test]
    fn test_outgauge_without_id() {
        let mut input = BytesMut::new();
        input.extend_from_slice(&RAW);

        let mut buf = input.clone().freeze();

        let outgauge = Outgauge::decode(&mut buf).unwrap();
        assert_eq!(buf.remaining(), 0);

        let mut output = BytesMut::new();
        outgauge.encode(&mut output).unwrap();

        assert_eq!(
            output.as_ref(),
            input.as_ref(),
            "assert reads and writes. left=actual, right=expected"
        );

        assert_eq!(outgauge.car, Vehicle::Xrt);
        assert!(matches!(outgauge.gear, Gear::Gear(2)));
        assert_eq!(outgauge.plid, PlayerId(1));
        assert_eq!(outgauge.rpm, 2450.390625);
        assert_eq!(outgauge.turbo, -0.9999896883964539);
        assert_eq!(outgauge.engtemp, 0.0);
        assert_eq!(outgauge.display1.trim_end_matches(' '), "Fuel 99.9%");
        assert_eq!(outgauge.display2.trim_end_matches(' '), "Brake Bal Fr 75%");
    }

    #[test]
    fn test_outgauge_with_id() {
        let mut input = BytesMut::new();
        input.extend_from_slice(&RAW);
        input.put_i32_le(10);

        let mut buf = input.clone().freeze();

        let outgauge = Outgauge::decode(&mut buf).unwrap();
        assert_eq!(buf.remaining(), 0);
        assert!(matches!(outgauge.id, Some(OutgaugeId(10))));

        let mut output = BytesMut::new();
        outgauge.encode(&mut output).unwrap();

        assert_eq!(
            output.as_ref(),
            input.as_ref(),
            "assert reads and writes. left=actual, right=expected"
        );
    }
}
