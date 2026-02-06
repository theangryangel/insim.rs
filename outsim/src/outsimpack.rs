//! OutsimPack
use std::time::Duration;

use bytes::Buf;
use insim_core::{Decode, Encode, coordinate::Coordinate, vector::Vector};

use crate::OutsimId;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Outsim packet
pub struct OutsimPack {
    /// Time, useful for ordering
    pub time: Duration,

    /// Angular velocity
    pub angvel: Vector,

    /// Heading
    pub heading: f32,

    /// Pitch
    pub pitch: f32,

    /// Roll
    pub roll: f32,

    /// Acceleration
    pub accel: Vector,

    /// Velocity
    pub vel: Vector,

    /// Position
    pub pos: Coordinate,

    /// Optional identifier
    pub id: Option<OutsimId>,
}

impl Encode for OutsimPack {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        let time = self.time.as_millis();
        (time as u32).encode(buf)?;
        self.angvel.encode(buf)?;
        self.heading.encode(buf)?;
        self.pitch.encode(buf)?;
        self.roll.encode(buf)?;
        self.accel.encode(buf)?;
        self.vel.encode(buf)?;
        self.pos.encode(buf)?;
        if let Some(id) = self.id {
            id.encode(buf)?;
        }
        Ok(())
    }
}

impl Decode for OutsimPack {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let time = Duration::from_millis(u32::decode(buf)? as u64);
        let angvel = Vector::decode(buf)?;
        let heading = f32::decode(buf)?;
        let pitch = f32::decode(buf)?;
        let roll = f32::decode(buf)?;
        let accel = Vector::decode(buf)?;
        let vel = Vector::decode(buf)?;
        let pos = Coordinate::decode(buf)?;
        let id = if buf.has_remaining() {
            Some(OutsimId::decode(buf)?)
        } else {
            None
        };

        Ok(Self {
            time,
            angvel,
            heading,
            pitch,
            roll,
            accel,
            vel,
            pos,
            id,
        })
    }
}

#[cfg(test)]
mod test {
    use bytes::{BufMut, BytesMut};

    use super::*;

    const RAW: [u8; 64] = [
        240, // Time (1)
        50,  // Time (2)
        0,   // Time (3)
        0,   // Time (4)
        155, // AngVelX (1)
        155, // AngVelX (2)
        12,  // AngVelX (3)
        60,  // AngVelX (4)
        180, // AngVelY (1)
        252, // AngVelY (2)
        109, // AngVelY (3)
        188, // AngVelY (4)
        149, // AngVelZ (1)
        60,  // AngVelZ (2)
        47,  // AngVelZ (3)
        60,  // AngVelZ (4)
        23,  // Heading (1)
        119, // Heading (2)
        134, // Heading (3)
        62,  // Heading (4)
        9,   // Pitch (1)
        32,  // Pitch (2)
        225, // Pitch (3)
        60,  // Pitch (4)
        84,  // Roll (1)
        42,  // Roll (2)
        63,  // Roll (3)
        186, // Roll (4)
        118, // AccelX (1)
        69,  // AccelX (2)
        154, // AccelX (3)
        191, // AccelX (4)
        150, // AccelY (1)
        84,  // AccelY (2)
        136, // AccelY (3)
        64,  // AccelY (4)
        148, // AccelZ (1)
        155, // AccelZ (2)
        51,  // AccelZ (3)
        62,  // AccelZ (4)
        64,  // VelX (1)
        200, // VelX (2)
        128, // VelX (3)
        192, // VelX (4)
        21,  // VelY (1)
        143, // VelY (2)
        111, // VelY (3)
        65,  // VelY (4)
        106, // VelZ (1)
        9,   // VelZ (2)
        193, // VelZ (3)
        187, // VelZ (4)
        35,  // PosX (1)
        134, // PosX (2)
        62,  // PosX (3)
        253, // PosX (4)
        166, // PosY (1)
        226, // PosY (2)
        163, // PosY (3)
        248, // PosY (4)
        42,  // PosZ (1)
        26,  // PosZ (2)
        2,   // PosZ (3)
        0,   // PosZ (4)
    ];

    #[test]
    fn test_outsim_without_id() {
        let mut input = BytesMut::new();
        input.extend_from_slice(&RAW);

        let mut buf = input.clone().freeze();

        let outsim = OutsimPack::decode(&mut buf).unwrap();
        assert_eq!(buf.remaining(), 0);

        let mut output = BytesMut::new();
        outsim.encode(&mut output).unwrap();

        assert_eq!(
            output.as_ref(),
            input.as_ref(),
            "assert reads and writes. left=actual, right=expected"
        );
    }

    #[test]
    fn test_outsim_with_id() {
        let mut input = BytesMut::new();
        input.extend_from_slice(&RAW);
        input.put_i32_le(10);

        let mut buf = input.clone().freeze();

        let outsim = OutsimPack::decode(&mut buf).unwrap();
        assert_eq!(buf.remaining(), 0);
        assert!(matches!(outsim.id, Some(OutsimId(10))));

        let mut output = BytesMut::new();
        outsim.encode(&mut output).unwrap();

        assert_eq!(
            output.as_ref(),
            input.as_ref(),
            "assert reads and writes. left=actual, right=expected"
        );
    }
}
