use bitflags::bitflags;
use insim_core::{point::Point, ReadWriteBuf};

use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct OsMain {
    pub angvel: (f32, f32, f32),

    pub heading: f32,

    pub pitch: f32,

    pub roll: f32,

    pub accel: Point<f32>,

    pub vel: Point<f32>,

    pub pos: Point<i32>,
}

impl ReadWriteBuf for OsMain {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let angvel = (
            f32::read_buf(buf)?,
            f32::read_buf(buf)?,
            f32::read_buf(buf)?,
        );
        let heading = f32::read_buf(buf)?;
        let pitch = f32::read_buf(buf)?;
        let roll = f32::read_buf(buf)?;
        let accel = Point::<f32>::read_buf(buf)?;
        let vel = Point::<f32>::read_buf(buf)?;
        let pos = Point::<i32>::read_buf(buf)?;
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

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.angvel.0.write_buf(buf)?;
        self.angvel.1.write_buf(buf)?;
        self.angvel.2.write_buf(buf)?;
        self.heading.write_buf(buf)?;
        self.pitch.write_buf(buf)?;
        self.roll.write_buf(buf)?;
        self.accel.write_buf(buf)?;
        self.vel.write_buf(buf)?;
        self.pos.write_buf(buf)?;
        Ok(())
    }
}

bitflags! {
    /// Flags for AI Detection
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    pub struct AiFlags: u8 {
        /// Detect if engine running
        const IGNITION = (1 << 0);
        /// Upshift currently held
        const CHUP = (1 << 2);
        /// Downshift currently held
        const CHDN = (1 << 3);
    }
}

impl_bitflags_from_to_bytes!(AiFlags, u8);

bitflags! {
    /// Flags for AI Detection
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    pub struct AiShowLights: u32 {
        /// Shift light
        const SHIFT = 1;
        /// Fullbeam
        const FULLBEAM = (1 << 1);
        /// Handbrake
        const HANDBRAKE = (1 << 2);
        /// Pitspeed limiter
        const PITSPEED = (1 << 3);
        /// Traction control
        const TC = (1 << 4);
        /// Left turn
        const SIGNAL_L = (1 << 5);
        /// Right turn
        const SIGNAL_R = (1 << 6);
        /// Hazards
        const SIGNAL_ANY = (1 << 7);
        /// Oil pressure warning
        const OILWARN = (1 << 8);
        /// Battery warning
        const BATTERY = (1 << 9);
        /// ABS
        const ABS = (1 << 10);
        /// Engine damage
        const ENGINE = (1 << 11);
        /// Rear fog lights
        const FOG_REAR = (1 << 12);
        /// Front fog lights
        const FOG_FRONT = (1 << 13);
        /// Dipped headlights
        const DIPPED = (1 << 14);
        /// Low fuel warning
        const FUELWARN = (1 << 15);
        /// Sidelights
        const SIDELIGHTS = (1 << 16);
        /// Neutral
        const NEUTRAL = (1 << 17);
    }
}

impl_bitflags_from_to_bytes!(AiShowLights, u32);

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// AI Info
pub struct Aii {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Set to choose 16-bit
    pub plid: PlayerId,

    /// Outsim main packet
    pub osmain: OsMain,

    /// Flags
    pub flags: AiFlags,

    #[read_write_buf(pad_after = 2)]
    /// Current gear
    pub gear: u8,

    #[read_write_buf(pad_after = 8)]
    /// Current RPM
    pub rpm: f32,

    #[read_write_buf(pad_after = 12)]
    /// Current lights
    pub showlights: AiShowLights,
}

impl_typical_with_request_id!(Aii);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_aii() {
        assert_from_to_bytes!(
            Aii,
            [
                1,   // reqi
                3,   // plid
                126, // osmain.angvelx (1)
                231, // osmain.angvelx (2)
                140, // osmain.angvelx (3)
                188, // osmain.angvelx (4)
                0,   // osmain.angvely (1)
                70,  // osmain.angvely (2)
                13,  // osmain.angvely (3)
                188, // osmain.angvely (4)
                130, // osmain.angvelz (1)
                122, // osmain.angvelz (2)
                165, // osmain.angvelz (3)
                187, // osmain.angvelz (4)
                138, // osmain.heading (1)
                10,  // osmain.heading (2)
                83,  // osmain.heading (3)
                182, // osmain.heading (4)
                248, // osmain.pitch (1)
                153, // osmain.pitch (2)
                138, // osmain.pitch (3)
                60,  // osmain.pitch (4)
                156, // osmain.roll (1)
                143, // osmain.roll (2)
                135, // osmain.roll (3)
                186, // osmain.roll (4)
                47,  // osmain.accelx (1)
                77,  // osmain.accelx (2)
                153, // osmain.accelx (3)
                57,  // osmain.accelx (4)
                159, // osmain.accely (1)
                58,  // osmain.accely (2)
                102, // osmain.accely (3)
                57,  // osmain.accely (4)
                222, // osmain.accelz (1)
                252, // osmain.accelz (2)
                251, // osmain.accelz (3)
                58,  // osmain.accelz (4)
                247, // osmain.velx (1)
                213, // osmain.velx (2)
                50,  // osmain.velx (3)
                183, // osmain.velx (4)
                34,  // osmain.vely (1)
                197, // osmain.vely (2)
                114, // osmain.vely (3)
                56,  // osmain.vely (4)
                218, // osmain.velz (1)
                126, // osmain.velz (2)
                10,  // osmain.velz (3)
                56,  // osmain.velz (4)
                249, // osmain.posx (1)
                255, // osmain.posx (2)
                79,  // osmain.posx (3)
                253, // osmain.posx (4)
                161, // osmain.posy (1)
                255, // osmain.posy (2)
                97,  // osmain.posy (3)
                248, // osmain.posy (4)
                185, // osmain.posz (1)
                26,  // osmain.posz (2)
                2,   // osmain.posz (3)
                0,   // osmain.posz (4)
                5,   // flags
                2,   // gear
                0,   // sp0
                0,   // sp1
                255, 0, 0, 0, // rpm
                0, 0, 0, 0, // spf0
                0, 0, 0, 0, // spf1
                1, 0, 0, 0, // showlights
                0, 0, 0, 0, // spu1
                0, 0, 0, 0, // spu2
                0, 0, 0, 0, // spu2
            ],
            |parsed: Aii| {
                assert_eq!(parsed.gear, 2);
                assert!(parsed.showlights.contains(AiShowLights::SHIFT));
            }
        );
    }
}
