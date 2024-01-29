use insim_core::binrw::{self, binrw};

use crate::identifiers::{ConnectionId, RequestId};

/// Used within the [Axm] packet.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ObjectInfo {
    /// X coordinate of object
    pub x: i16,
    /// Y coordinate of object
    pub y: i16,
    /// Z coordinate of object
    pub z: u8,

    /// Flags
    pub flags: u8,

    /// Index of object
    pub index: u8,

    /// Heading/direction of object
    pub heading: u8,
}

/// Actions that can be taken as part of [Axm].
#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
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
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    /// AutoX object flags
    pub struct PmoFlags: u16 {
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

/// AutoX Multiple Objects - Report on/add/remove multiple AutoX objects
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Axm {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Number of objects in this packet
    #[bw(calc = info.len() as u8)]
    pub numo: u8,

    /// Unique id of the connection that sent the packet
    pub ucid: ConnectionId,

    /// Action that was taken
    pub pmoaction: PmoAction,

    /// Bitflags providing additional information about what has happened, or what you want to
    /// happen
    #[brw(pad_after = 1)]
    pub pmoflags: PmoFlags,

    /// List of information about the affected objects
    #[br(count = numo)]
    pub info: Vec<ObjectInfo>,
}
