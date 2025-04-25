use bitflags::bitflags;
use insim_core::{
    binrw::{self, binrw},
    point::Point,
    ReadWriteBuf,
};

use crate::identifiers::{PlayerId, RequestId};

#[binrw]
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
    #[binrw]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
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

#[binrw]
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

    #[brw(pad_after = 2)]
    #[read_write_buf(pad_after = 2)]
    /// Current gear
    pub gear: u8,

    #[brw(pad_after = 8)]
    #[read_write_buf(pad_after = 8)]
    /// Current RPM
    pub rpm: f32,

    #[brw(pad_after = 12)]
    #[read_write_buf(pad_after = 12)]

    /// Current lights
    // FIXME: Needs translating into a bitflags implementation
    pub showlights: u32,
}

impl_typical_with_request_id!(Aii);
