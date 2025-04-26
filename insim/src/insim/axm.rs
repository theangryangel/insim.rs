use bytes::{Buf, BufMut};
use insim_core::ReadWriteBuf;

use crate::identifiers::{ConnectionId, RequestId};

const AXM_MAX_OBJECTS: usize = 60;

/// Used within the [Axm] packet.
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ObjectInfo {
    /// X coordinate of object
    pub x: i16,
    /// Y coordinate of object
    pub y: i16,
    /// Z coordinate of object
    pub z: u8,

    // TODO: check layout if this has something we can do with it
    /// Flags
    pub flags: u8,

    /// Index of object
    pub index: u8,

    /// Heading/direction of object
    pub heading: u8,
}

/// Actions that can be taken as part of [Axm].
#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[non_exhaustive]
pub enum PmoAction {
    #[default]
    /// Sent by the layout loading system only
    LoadingFile = 0,

    /// Add objects
    AddObjects = 1,

    /// Delete objects
    DelObjects = 2,

    /// Remove/clear all objects
    ClearAll = 3,

    /// Indicates a reply to a TINY_AXM packet
    TinyAxm = 4,

    /// Indicates a reply to a TTC_SEL packet
    TtcSel = 5,

    /// Set a connection's layout editor selection
    Selection = 6,

    /// User pressed O without anything selected
    Position = 7,

    /// Request Z values / reply with Z values
    GetZ = 8,
}

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    /// AutoX object flags
    pub struct PmoFlags: u8 {
        /// LFS has reached the end of a layout file which it is loading. The added objects will then be optimised.
        const FILE_END = (1 << 0);

        /// When objects are moved or modified in the layout editor, two IS_AXM packets are
        /// sent.  A PMO_DEL_OBJECTS followed by a PMO_ADD_OBJECTS.  In this case the flag
        /// PMO_MOVE_MODIFY is set in the PMOFlags byte of both packets.
        const MOVE_MODIFY = (1 << 1);

        /// If you send an IS_AXM with PMOAction of PMO_SELECTION it is possible for it to be
        /// either a selection of real objects (as if the user selected several objects while
        /// holding the CTRL key) or a clipboard selection (as if the user pressed CTRL+C after
        /// selecting objects).  Clipboard is the default selection mode.  A real selection can
        /// be set by using the PMO_SELECTION_REAL bit in the PMOFlags byte.
        const SELECTION_REAL = (1 << 2);

        /// If you send an IS_AXM with PMOAction of PMO_ADD_OBJECTS you may wish to set the
        /// UCID to one of the guest connections (for example if that user's action caused the
        /// objects to be added).  In this case some validity checks are done on the guest's
        /// computer which may report "invalid position" or "intersecting object" and delete
        /// the objects.  This can be avoided by setting the PMO_AVOID_CHECK bit.
        const AVOID_CHECK = (1 << 3);
    }
}

impl_bitflags_from_to_bytes!(PmoFlags, u8);

/// AutoX Multiple Objects - Report on/add/remove multiple AutoX objects
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Axm {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique id of the connection that sent the packet
    pub ucid: ConnectionId,

    /// Action that was taken
    pub pmoaction: PmoAction,

    /// Bitflags providing additional information about what has happened, or what you want to
    /// happen
    pub pmoflags: PmoFlags,

    /// List of information about the affected objects
    pub info: Vec<ObjectInfo>,
}

impl_typical_with_request_id!(Axm);

impl ReadWriteBuf for Axm {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        let mut numo = u8::read_buf(buf)?;
        let ucid = ConnectionId::read_buf(buf)?;
        let pmoaction = PmoAction::read_buf(buf)?;
        let pmoflags = PmoFlags::read_buf(buf)?;
        buf.advance(1);
        let mut info = Vec::with_capacity(numo as usize);
        while numo > 0 {
            info.push(ObjectInfo::read_buf(buf)?);
            numo -= 1;
        }

        Ok(Self {
            reqi,
            ucid,
            pmoaction,
            pmoflags,
            info,
        })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        let len = self.info.len();
        if len > AXM_MAX_OBJECTS {
            return Err(insim_core::Error::TooLarge);
        }
        (len as u8).write_buf(buf)?;
        self.ucid.write_buf(buf)?;
        self.pmoaction.write_buf(buf)?;
        self.pmoflags.write_buf(buf)?;
        buf.put_bytes(0, 1);
        for i in self.info.iter() {
            i.write_buf(buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_axm() {
        assert_from_to_bytes!(
            Axm,
            [
                0,   // reqi
                2,   // numo
                3,   // ucid
                1,   // pmoaction
                4,   // pmoflags
                0,   // objects
                172, // info[1] - x (1)
                218, // info[1] - x (2)
                25,  // info[1] - y (1)
                136, // info[1] - y (2)
                8,   // info[1] - zbyte
                0,   // info[1] - flags
                1,   // info[1] - objectindex
                128, // info[1] - heading
                172, // info[2] - x (1)
                218, // info[2] - x (2)
                25,  // info[2] - y (1)
                136, // info[2] - y (2)
                8,   // info[2] - zbyte
                0,   // info[2] - flags
                2,   // info[2] - objectindex
                128, // info[2] - heading
            ],
            |axm: Axm| {
                assert_eq!(axm.info.len(), 2);
                assert_eq!(axm.info[0].z, 8);
                assert_eq!(axm.info[0].flags, 0);
                assert_eq!(axm.info[0].index, 1);
                assert_eq!(axm.info[0].heading, 128);

                assert_eq!(axm.info[1].index, 2);
            }
        )
    }
}
