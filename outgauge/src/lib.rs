#![doc = include_str!("../README.md")]
#![cfg_attr(test, deny(warnings, unreachable_pub))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use std::{
    fmt,
    ops::{Deref, DerefMut},
    time::Duration,
};

pub use ::insim_core as core;
use bytes::Buf;
use insim_core::{
    Decode, DecodeContext, Encode, EncodeContext, dash_lights::DashLights, gear::Gear,
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
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.encode("bits", &self.bits())
    }
}

impl Decode for OutgaugeFlags {
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        ctx.decode::<u16>("bits").map(Self::from_bits_truncate)
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
    const PRIMITIVE: bool = true;
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        Ok(OutgaugeId(i32::decode(ctx)?))
    }
}

impl Encode for OutgaugeId {
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        self.0.encode(ctx)
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
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.encode_duration::<u32>("time", self.time)?;
        ctx.encode("car", &self.car)?;
        ctx.encode("flags", &self.flags)?;
        ctx.encode("gear", &self.gear)?;
        ctx.encode("plid", &self.plid)?;
        ctx.encode("speed", &self.speed.to_meters_per_sec())?;
        ctx.encode("rpm", &self.rpm)?;
        ctx.encode("turbo", &self.turbo)?;
        ctx.encode("engtemp", &self.engtemp)?;
        ctx.encode("fuel", &self.fuel)?;
        ctx.encode("oilpressure", &self.oilpressure)?;
        ctx.encode("oiltemp", &self.oiltemp)?;
        ctx.encode("dashlights", &self.dashlights)?;
        ctx.encode("showlights", &self.showlights)?;
        ctx.encode("throttle", &self.throttle)?;
        ctx.encode("clutch", &self.clutch)?;
        ctx.encode("brake", &self.brake)?;
        ctx.encode_ascii("display1", &self.display1, 16, false)?;
        ctx.encode_ascii("display2", &self.display2, 16, false)?;
        if let Some(id) = self.id {
            ctx.encode("id", &id)?;
        }
        Ok(())
    }
}

impl Decode for Outgauge {
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let time = ctx.decode_duration::<u32>("time")?;
        let car = ctx.decode::<Vehicle>("car")?;
        let flags = ctx.decode::<OutgaugeFlags>("flags")?;
        let gear = ctx.decode::<Gear>("gear")?;
        let plid = ctx.decode::<PlayerId>("plid")?;
        let speed = Speed::from_meters_per_sec(ctx.decode::<f32>("speed")?);
        let rpm = ctx.decode::<f32>("rpm")?;
        let turbo = ctx.decode::<f32>("turbo")?;
        let engtemp = ctx.decode::<f32>("engtemp")?;
        let fuel = ctx.decode::<f32>("fuel")?;
        let oilpressure = ctx.decode::<f32>("oilpressure")?;
        let oiltemp = ctx.decode::<f32>("oiltemp")?;

        let dashlights = ctx.decode::<DashLights>("dashlights")?;
        let showlights = ctx.decode::<DashLights>("showlights")?;

        let throttle = ctx.decode::<f32>("throttle")?;
        let clutch = ctx.decode::<f32>("clutch")?;
        let brake = ctx.decode::<f32>("brake")?;

        let display1 = ctx.decode_ascii("display1", 16)?;
        let display2 = ctx.decode_ascii("display2", 16)?;

        let id = if ctx.buf.has_remaining() {
            Some(ctx.decode::<OutgaugeId>("id")?)
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
    use bytes::{BufMut, Buf, BytesMut};
    use insim_core::{DecodeContext, EncodeContext};

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

        let outgauge = Outgauge::decode(&mut DecodeContext::new(&mut buf)).unwrap();
        assert_eq!(buf.remaining(), 0);

        let mut output = BytesMut::new();
        outgauge.encode(&mut EncodeContext::new(&mut output)).unwrap();

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

        let outgauge = Outgauge::decode(&mut DecodeContext::new(&mut buf)).unwrap();
        assert_eq!(buf.remaining(), 0);
        assert!(matches!(outgauge.id, Some(OutgaugeId(10))));

        let mut output = BytesMut::new();
        outgauge.encode(&mut EncodeContext::new(&mut output)).unwrap();

        assert_eq!(
            output.as_ref(),
            input.as_ref(),
            "assert reads and writes. left=actual, right=expected"
        );
    }
}
