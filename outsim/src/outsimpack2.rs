//! OutsimPack
use std::time::Duration;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use insim_core::{
    gear::Gear, point::Point, Decode, DecodeError, DecodeString, Encode, EncodeError, EncodeString,
};

use crate::OutsimId;

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    /// Describes the setup of a player and the various helpers that may be enabled, such as
    /// auto-clutch, etc.
    pub struct OutSimOpts: u16 {
        /// Header
        const HEADER = 1;
        /// ID
        const ID = (1 << 1);
        /// Time
        const TIME = (1 << 2);
        /// Main
        const MAIN = (1 << 3);
        /// Inputs
        const INPUTS = (1 << 4);
        /// Drive
        const DRIVE = (1 << 5);
        /// Distance
        const DISTANCE = (1 << 6);
        /// Wheels
        const WHEELS = (1 << 7);
        /// Main
        const EXTRA = (1 << 8);
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// OutsimMain packet
pub struct OutsimMain {
    /// Angular velocity
    pub angvel: (f32, f32, f32),

    /// Heading
    pub heading: f32,

    /// Pitch
    pub pitch: f32,

    /// Roll
    pub roll: f32,

    /// Acceleration
    pub accel: (f32, f32, f32),

    /// Velocity
    pub vel: (f32, f32, f32),

    /// Position
    pub pos: Point<i32>,
}

impl Encode for OutsimMain {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.angvel.0.encode(buf)?;
        self.angvel.1.encode(buf)?;
        self.angvel.2.encode(buf)?;
        self.heading.encode(buf)?;
        self.pitch.encode(buf)?;
        self.roll.encode(buf)?;
        self.accel.0.encode(buf)?;
        self.accel.1.encode(buf)?;
        self.accel.2.encode(buf)?;
        self.vel.0.encode(buf)?;
        self.vel.1.encode(buf)?;
        self.vel.2.encode(buf)?;
        self.pos.encode(buf)?;
        Ok(())
    }
}

impl Decode for OutsimMain {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let angvel = (f32::decode(buf)?, f32::decode(buf)?, f32::decode(buf)?);
        let heading = f32::decode(buf)?;
        let pitch = f32::decode(buf)?;
        let roll = f32::decode(buf)?;
        let accel = (f32::decode(buf)?, f32::decode(buf)?, f32::decode(buf)?);
        let vel = (f32::decode(buf)?, f32::decode(buf)?, f32::decode(buf)?);
        let pos = Point::decode(buf)?;

        Ok(Self {
            angvel,
            heading,
            pitch,
            roll,
            accel,
            vel,
            pos,
        })
    }
}

#[derive(Debug, Clone, Copy, Encode, Decode, Default)]
/// Outsim Inputs, used as part of OutsimPack2
pub struct OutsimInputs {
    /// Throttle, 0-1
    pub throttle: f32,
    /// Brake, 0-1
    pub brake: f32,
    /// Input steer, radians
    pub inputsteer: f32,
    /// Clutch, 0-1
    pub clutch: f32,
    /// Handbrake, 0-1
    pub handbrake: f32,
}

#[derive(Debug, Clone, Copy, Encode, Decode, Default)]
/// Outsim Inputs, used as part of OutsimPack2
pub struct OutsimWheel {
    /// Compression, from unloaded
    pub suspdeflect: f32,
    /// Including Ackermann and toe
    pub steer: f32,
    /// Force right
    pub xforce: f32,
    /// Force forward
    pub yforce: f32,
    /// Perpendicular to surface
    pub verticalload: f32,
    /// Radians/s
    pub angvel: f32,
    /// Radians a-c viewed from rear
    pub leanreltoroad: f32,
    /// Celcius
    pub airtemp: u8,
    /// Slip friction, 0-255
    pub slipfraction: u8,
    #[insim(pad_after = 1)]
    /// Touching ground
    pub touching: bool,
    /// Slip ratio
    pub slipratio: f32,
    /// Tangent of slip angle
    pub tanslipangle: f32,
}

#[derive(Debug, Default)]
#[allow(missing_docs)]
/// OutsimPack2
pub struct OutsimPack2 {
    // if OutSimOpts.HEADER
    pub header: Option<String>,
    // endif

    // if OutSimOpts.ID
    pub id: Option<OutsimId>,
    // endif

    // if OutSimOpts.TIME
    pub time: Option<Duration>,
    // endif

    // if OutSimOpts.MAIN
    pub osmain: Option<OutsimMain>,
    // endif

    // if OutSimOpts.INPUTS
    pub osinputs: Option<OutsimInputs>,
    // endif

    // if OutSimOpts.DRIVE
    pub gear: Option<Gear>,
    pub engineangvel: Option<f32>,
    pub maxtorqueatvel: Option<f32>,
    // endif

    // if OutSimOpts.DISTANCE
    pub currentlapdist: Option<f32>,
    pub indexeddistance: Option<f32>,
    // endif

    // if OutSimOpts.WHEELS
    pub oswheels: Option<[OutsimWheel; 4]>,
    // endif

    // if OutSimOpts.EXTRA
    pub steertorque: Option<f32>,
    // endif
}

impl OutsimPack2 {
    /// How should we decode this?
    pub fn decode_with_options(buf: &mut Bytes, opts: &OutSimOpts) -> Result<Self, DecodeError> {
        let mut val = Self::default();
        if opts.contains(OutSimOpts::HEADER) {
            val.header = Some(String::decode_ascii(buf, 4)?);
        }

        if opts.contains(OutSimOpts::ID) {
            val.id = Some(OutsimId::decode(buf)?);
        }

        if opts.contains(OutSimOpts::TIME) {
            let time = Duration::from_millis(u32::decode(buf)? as u64);
            val.time = Some(time);
        }

        if opts.contains(OutSimOpts::MAIN) {
            val.osmain = Some(OutsimMain::decode(buf)?);
        }

        if opts.contains(OutSimOpts::INPUTS) {
            val.osinputs = Some(OutsimInputs::decode(buf)?);
        }

        if opts.contains(OutSimOpts::DRIVE) {
            val.gear = Some(Gear::decode(buf)?);
            buf.advance(3);
            val.engineangvel = Some(f32::decode(buf)?);
            val.maxtorqueatvel = Some(f32::decode(buf)?);
        }

        if opts.contains(OutSimOpts::DISTANCE) {
            val.currentlapdist = Some(f32::decode(buf)?);
            val.indexeddistance = Some(f32::decode(buf)?);
        }

        if opts.contains(OutSimOpts::WHEELS) {
            val.oswheels = Some(<[OutsimWheel; 4]>::decode(buf)?);
        }

        if opts.contains(OutSimOpts::EXTRA) {
            val.steertorque = Some(f32::decode(buf)?);
            buf.advance(4);
        }

        Ok(val)
    }

    /// How should we encode this?
    pub fn encode_with_options(
        &self,
        buf: &mut BytesMut,
        opts: &OutSimOpts,
    ) -> Result<(), EncodeError> {
        if opts.contains(OutSimOpts::HEADER) {
            if let Some(header) = &self.header {
                header.encode_ascii(buf, 4, false)?;
            } else {
                buf.put_bytes(0, 4);
            }
        }

        if opts.contains(OutSimOpts::ID) {
            self.id.unwrap_or_default().encode(buf)?;
        }

        if opts.contains(OutSimOpts::TIME) {
            (self.time.unwrap_or_default().as_millis() as u32).encode(buf)?;
        }

        if opts.contains(OutSimOpts::MAIN) {
            self.osmain.unwrap_or_default().encode(buf)?;
        }

        if opts.contains(OutSimOpts::INPUTS) {
            self.osinputs.unwrap_or_default().encode(buf)?;
        }

        if opts.contains(OutSimOpts::DRIVE) {
            self.gear.unwrap_or_default().encode(buf)?;
            buf.put_bytes(0, 3);
            self.engineangvel.unwrap_or_default().encode(buf)?;
            self.maxtorqueatvel.unwrap_or_default().encode(buf)?;
        }

        if opts.contains(OutSimOpts::DISTANCE) {
            self.currentlapdist.unwrap_or_default().encode(buf)?;
            self.indexeddistance.unwrap_or_default().encode(buf)?;
        }

        if opts.contains(OutSimOpts::WHEELS) {
            self.oswheels.unwrap_or_default().encode(buf)?;
        }

        if opts.contains(OutSimOpts::EXTRA) {
            self.steertorque.unwrap_or_default().encode(buf)?;
            buf.put_bytes(0, 4);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header_only() {
        let mut input = BytesMut::new();
        input.extend_from_slice(&[b'L', b'F', b'S', b'T']);

        let opts = OutSimOpts::HEADER;

        let buf = input.freeze();

        let parsed = OutsimPack2::decode_with_options(&mut buf.clone(), &opts).unwrap();
        assert!(parsed.header.is_some());

        let mut output = BytesMut::new();

        parsed.encode_with_options(&mut output, &opts).unwrap();

        assert_eq!(buf.as_ref(), output.as_ref());
    }

    #[test]
    fn test_id_only() {
        let mut input = BytesMut::new();
        input.extend_from_slice(&[64, 4, 5, 7]);

        let opts = OutSimOpts::ID;

        let buf = input.freeze();

        let parsed = OutsimPack2::decode_with_options(&mut buf.clone(), &opts).unwrap();
        assert!(parsed.header.is_none());
        assert!(matches!(parsed.id, Some(OutsimId(117769280))));

        let mut output = BytesMut::new();

        parsed.encode_with_options(&mut output, &opts).unwrap();

        assert_eq!(buf.as_ref(), output.as_ref());
    }

    #[test]
    fn test_kitchen_sink() {
        let mut input = BytesMut::new();
        input.extend_from_slice(&[
            76,  // Header (1)
            70,  // Header (2)
            83,  // Header (3)
            84,  // Header (4)
            64,  // ID (1)
            4,   // ID (2)
            5,   // ID (3)
            7,   // ID (4)
            240, // Time (1)
            50,  // Time (2)
            0,   // Time (3)
            0,   // Time (4)
            126, // AngVelX (1)
            231, // AngVelX (2)
            140, // AngVelX (3)
            188, // AngVelX (4)
            0,   // AngVelY (1)
            70,  // AngVelY (2)
            13,  // AngVelY (3)
            188, // AngVelY (4)
            130, // AngVelZ (1)
            122, // AngVelZ (2)
            165, // AngVelZ (3)
            187, // AngVelZ (4)
            138, // Heading (1)
            10,  // Heading (2)
            83,  // Heading (3)
            182, // Heading (4)
            248, // Pitch (1)
            153, // Pitch (2)
            138, // Pitch (3)
            60,  // Pitch (4)
            156, // Roll (1)
            143, // Roll (2)
            135, // Roll (3)
            186, // Roll (4)
            47,  // AccelX (1)
            77,  // AccelX (2)
            153, // AccelX (3)
            57,  // AccelX (4)
            159, // AccelY (1)
            58,  // AccelY (2)
            102, // AccelY (3)
            57,  // AccelY (4)
            222, // AccelZ (1)
            252, // AccelZ (2)
            251, // AccelZ (3)
            58,  // AccelZ (4)
            247, // VelX (1)
            213, // VelX (2)
            50,  // VelX (3)
            183, // VelX (4)
            34,  // VelY (1)
            197, // VelY (2)
            114, // VelY (3)
            56,  // VelY (4)
            218, // VelZ (1)
            126, // VelZ (2)
            10,  // VelZ (3)
            56,  // VelZ (4)
            249, // PosX (1)
            255, // PosX (2)
            79,  // PosX (3)
            253, // PosX (4)
            161, // PosY (1)
            255, // PosY (2)
            97,  // PosY (3)
            248, // PosY (4)
            185, // PosZ (1)
            26,  // PosZ (2)
            2,   // PosZ (3)
            0,   // PosZ (4)
            184, // Throttle (1)
            81,  // Throttle (2)
            56,  // Throttle (3)
            63,  // Throttle (4)
            164, // Brake (1)
            216, // Brake (2)
            35,  // Brake (3)
            62,  // Brake (4)
            108, // InputSteer (1)
            188, // InputSteer (2)
            156, // InputSteer (3)
            62,  // InputSteer (4)
            248, // Clutch (1)
            240, // Clutch (2)
            90,  // Clutch (3)
            63,  // Clutch (4)
            0,   // Handbrake (1)
            0,   // Handbrake (2)
            128, // Handbrake (3)
            63,  // Handbrake (4)
            3,   // Gear
            0,   // Sp1
            0,   // Sp2
            0,   // Sp3
            12,  // EngineAngVel (1)
            9,   // EngineAngVel (2)
            209, // EngineAngVel (3)
            67,  // EngineAngVel (4)
            73,  // MaxTorqueAtVel (1)
            168, // MaxTorqueAtVel (2)
            172, // MaxTorqueAtVel (3)
            67,  // MaxTorqueAtVel (4)
            196, // CurrentLapDist (1)
            218, // CurrentLapDist (2)
            246, // CurrentLapDist (3)
            66,  // CurrentLapDist (4)
            119, // IndexedDistance (1)
            111, // IndexedDistance (2)
            245, // IndexedDistance (3)
            66,  // IndexedDistance (4)
            73,  // OSWheels[0] - SuspDeflect (1)
            105, // OSWheels[0] - SuspDeflect (2)
            203, // OSWheels[0] - SuspDeflect (3)
            61,  // OSWheels[0] - SuspDeflect (4)
            137, // OSWheels[0] - Steer (1)
            195, // OSWheels[0] - Steer (2)
            100, // OSWheels[0] - Steer (3)
            187, // OSWheels[0] - Steer (4)
            119, // OSWheels[0] - XForce (1)
            195, // OSWheels[0] - XForce (2)
            149, // OSWheels[0] - XForce (3)
            69,  // OSWheels[0] - XForce (4)
            145, // OSWheels[0] - YForce (1)
            221, // OSWheels[0] - YForce (2)
            91,  // OSWheels[0] - YForce (3)
            69,  // OSWheels[0] - YForce (4)
            194, // OSWheels[0] - VerticalLoad (1)
            102, // OSWheels[0] - VerticalLoad (2)
            150, // OSWheels[0] - VerticalLoad (3)
            69,  // OSWheels[0] - VerticalLoad (4)
            132, // OSWheels[0] - AngVel (1)
            235, // OSWheels[0] - AngVel (2)
            140, // OSWheels[0] - AngVel (3)
            66,  // OSWheels[0] - AngVel (4)
            59,  // OSWheels[0] - LeanRelToRoad (1)
            242, // OSWheels[0] - LeanRelToRoad (2)
            242, // OSWheels[0] - LeanRelToRoad (3)
            60,  // OSWheels[0] - LeanRelToRoad (4)
            40,  // OSWheels[0] - AirTemp
            255, // OSWheels[0] - SlipFraction
            1,   // OSWheels[0] - Touching
            0,   // OSWheels[0] - Sp3
            204, // OSWheels[0] - SlipRatio (1)
            118, // OSWheels[0] - SlipRatio (2)
            45,  // OSWheels[0] - SlipRatio (3)
            61,  // OSWheels[0] - SlipRatio (4)
            5,   // OSWheels[0] - TanSlipAngle (1)
            141, // OSWheels[0] - TanSlipAngle (2)
            252, // OSWheels[0] - TanSlipAngle (3)
            61,  // OSWheels[0] - TanSlipAngle (4)
            221, // OSWheels[1] - SuspDeflect (1)
            10,  // OSWheels[1] - SuspDeflect (2)
            111, // OSWheels[1] - SuspDeflect (3)
            61,  // OSWheels[1] - SuspDeflect (4)
            137, // OSWheels[1] - Steer (1)
            195, // OSWheels[1] - Steer (2)
            100, // OSWheels[1] - Steer (3)
            59,  // OSWheels[1] - Steer (4)
            52,  // OSWheels[1] - XForce (1)
            245, // OSWheels[1] - XForce (2)
            201, // OSWheels[1] - XForce (3)
            68,  // OSWheels[1] - XForce (4)
            33,  // OSWheels[1] - YForce (1)
            186, // OSWheels[1] - YForce (2)
            8,   // OSWheels[1] - YForce (3)
            69,  // OSWheels[1] - YForce (4)
            80,  // OSWheels[1] - VerticalLoad (1)
            151, // OSWheels[1] - VerticalLoad (2)
            12,  // OSWheels[1] - VerticalLoad (3)
            69,  // OSWheels[1] - VerticalLoad (4)
            231, // OSWheels[1] - AngVel (1)
            104, // OSWheels[1] - AngVel (2)
            141, // OSWheels[1] - AngVel (3)
            66,  // OSWheels[1] - AngVel (4)
            69,  // OSWheels[1] - LeanRelToRoad (1)
            12,  // OSWheels[1] - LeanRelToRoad (2)
            133, // OSWheels[1] - LeanRelToRoad (3)
            61,  // OSWheels[1] - LeanRelToRoad (4)
            40,  // OSWheels[1] - AirTemp
            255, // OSWheels[1] - SlipFraction
            1,   // OSWheels[1] - Touching
            0,   // OSWheels[1] - Sp3
            215, // OSWheels[1] - SlipRatio (1)
            172, // OSWheels[1] - SlipRatio (2)
            197, // OSWheels[1] - SlipRatio (3)
            61,  // OSWheels[1] - SlipRatio (4)
            69,  // OSWheels[1] - TanSlipAngle (1)
            42,  // OSWheels[1] - TanSlipAngle (2)
            250, // OSWheels[1] - TanSlipAngle (3)
            61,  // OSWheels[1] - TanSlipAngle (4)
            43,  // OSWheels[2] - SuspDeflect (1)
            80,  // OSWheels[2] - SuspDeflect (2)
            116, // OSWheels[2] - SuspDeflect (3)
            61,  // OSWheels[2] - SuspDeflect (4)
            68,  // OSWheels[2] - Steer (1)
            253, // OSWheels[2] - Steer (2)
            166, // OSWheels[2] - Steer (3)
            189, // OSWheels[2] - Steer (4)
            179, // OSWheels[2] - XForce (1)
            55,  // OSWheels[2] - XForce (2)
            161, // OSWheels[2] - XForce (3)
            69,  // OSWheels[2] - XForce (4)
            93,  // OSWheels[2] - YForce (1)
            225, // OSWheels[2] - YForce (2)
            179, // OSWheels[2] - YForce (3)
            195, // OSWheels[2] - YForce (4)
            22,  // OSWheels[2] - VerticalLoad (1)
            250, // OSWheels[2] - VerticalLoad (2)
            151, // OSWheels[2] - VerticalLoad (3)
            69,  // OSWheels[2] - VerticalLoad (4)
            190, // OSWheels[2] - AngVel (1)
            198, // OSWheels[2] - AngVel (2)
            132, // OSWheels[2] - AngVel (3)
            66,  // OSWheels[2] - AngVel (4)
            193, // OSWheels[2] - LeanRelToRoad (1)
            113, // OSWheels[2] - LeanRelToRoad (2)
            229, // OSWheels[2] - LeanRelToRoad (3)
            60,  // OSWheels[2] - LeanRelToRoad (4)
            40,  // OSWheels[2] - AirTemp
            198, // OSWheels[2] - SlipFraction
            1,   // OSWheels[2] - Touching
            0,   // OSWheels[2] - Sp3
            88,  // OSWheels[2] - SlipRatio (1)
            254, // OSWheels[2] - SlipRatio (2)
            173, // OSWheels[2] - SlipRatio (3)
            186, // OSWheels[2] - SlipRatio (4)
            195, // OSWheels[2] - TanSlipAngle (1)
            203, // OSWheels[2] - TanSlipAngle (2)
            1,   // OSWheels[2] - TanSlipAngle (3)
            62,  // OSWheels[2] - TanSlipAngle (4)
            65,  // OSWheels[2] - SuspDeflect (1)
            185, // OSWheels[2] - SuspDeflect (2)
            186, // OSWheels[2] - SuspDeflect (3)
            60,  // OSWheels[2] - SuspDeflect (4)
            166, // OSWheels[2] - Steer (1)
            175, // OSWheels[2] - Steer (2)
            171, // OSWheels[2] - Steer (3)
            189, // OSWheels[2] - Steer (4)
            255, // OSWheels[2] - XForce (1)
            218, // OSWheels[2] - XForce (2)
            119, // OSWheels[2] - XForce (3)
            67,  // OSWheels[2] - XForce (4)
            9,   // OSWheels[2] - YForce (1)
            13,  // OSWheels[2] - YForce (2)
            223, // OSWheels[2] - YForce (3)
            193, // OSWheels[2] - YForce (4)
            232, // OSWheels[2] - VerticalLoad (1)
            240, // OSWheels[2] - VerticalLoad (2)
            138, // OSWheels[2] - VerticalLoad (3)
            65,  // OSWheels[2] - VerticalLoad (4)
            217, // OSWheels[2] - AngVel (1)
            135, // OSWheels[2] - AngVel (2)
            122, // OSWheels[2] - AngVel (3)
            66,  // OSWheels[2] - AngVel (4)
            43,  // OSWheels[2] - LeanRelToRoad (1)
            2,   // OSWheels[2] - LeanRelToRoad (2)
            72,  // OSWheels[2] - LeanRelToRoad (2)
            61,  // OSWheels[2] - LeanRelToRoad (4)
            40,  // OSWheels[2] - AirTemp
            255, // OSWheels[2] - SlipFraction
            1,   // OSWheels[2] - Touching
            0,   // OSWheels[2] - Sp3
            54,  // OSWheels[2] - SlipRatio (1)
            59,  // OSWheels[2] - SlipRatio (2)
            55,  // OSWheels[2] - SlipRatio (3)
            188, // OSWheels[2] - SlipRatio (4)
            95,  // OSWheels[2] - TanSlipAngle (1)
            86,  // OSWheels[2] - TanSlipAngle (2)
            7,   // OSWheels[2] - TanSlipAngle (3)
            62,  // OSWheels[2] - TanSlipAngle (4)
            45,  // SteerTorque (1)
            155, // SteerTorque (2)
            16,  // SteerTorque (3)
            195, // SteerTorque (4)
            0,   // Spare (1)
            0,   // Spare (2)
            0,   // Spare (3)
            0,   // Spare (4)
        ]);

        let opts = OutSimOpts::all();

        let buf = input.freeze();

        let mut buf_to_parse = buf.clone();

        let parsed = OutsimPack2::decode_with_options(&mut buf_to_parse, &opts).unwrap();

        assert!(
            !buf_to_parse.has_remaining(),
            "Should have no remaining bytes: found {}",
            buf_to_parse.remaining()
        );

        assert!(parsed.header.is_some());
        assert!(matches!(parsed.id, Some(OutsimId(117769280))));

        let mut output = BytesMut::new();

        parsed.encode_with_options(&mut output, &opts).unwrap();

        assert_eq!(buf.as_ref(), output.as_ref());
    }
}
