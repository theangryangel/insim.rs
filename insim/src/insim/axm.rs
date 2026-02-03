use bytes::{Buf, BufMut};
use insim_core::{Decode, Encode};

use crate::identifiers::{ConnectionId, RequestId};

const AXM_MAX_OBJECTS: usize = 60;

pub use insim_core::object::ObjectInfo;

/// Actions that can be taken as part of [Axm].
#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[non_exhaustive]
pub enum PmoAction {
    #[default]
    /// Sent by the layout loading system only.
    LoadingFile = 0,

    /// Add objects.
    AddObjects = 1,

    /// Delete objects.
    DelObjects = 2,

    /// Remove/clear all objects.
    ClearAll = 3,

    /// Reply to [`TinyType::Axm`](crate::insim::TinyType::Axm).
    TinyAxm = 4,

    /// Reply to [`TtcType::Sel`](crate::insim::TtcType::Sel).
    TtcSel = 5,

    /// Set a connection's layout editor selection.
    Selection = 6,

    /// User pressed 'O' without anything selected.
    Position = 7,

    /// Request or reply with Z values.
    GetZ = 8,
}

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    /// AutoX object flags.
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

/// AutoX multiple objects update.
///
/// - Adds, removes, or reports layout objects.
/// - Carries a list of [ObjectInfo] entries.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Axm {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Connection that sent or requested the update.
    pub ucid: ConnectionId,

    /// Action taken or requested.
    pub pmoaction: PmoAction,

    /// Additional flags for the action.
    pub pmoflags: PmoFlags,

    /// Object list for the action.
    pub info: Vec<ObjectInfo>,
}

impl_typical_with_request_id!(Axm);

impl Decode for Axm {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf).map_err(|e| e.nested().context("Axm::reqi"))?;
        let mut numo = u8::decode(buf).map_err(|e| e.nested().context("Axm::numo"))?;
        let ucid = ConnectionId::decode(buf).map_err(|e| e.nested().context("Axm::ucid"))?;
        let pmoaction = PmoAction::decode(buf).map_err(|e| e.nested().context("Axm::pmoaction"))?;
        let pmoflags = PmoFlags::decode(buf).map_err(|e| e.nested().context("Axm::pmoflags"))?;
        buf.advance(1);
        let mut info = Vec::with_capacity(numo as usize);
        while numo > 0 {
            info.push(ObjectInfo::decode(buf).map_err(|e| e.nested().context("Axm::info"))?);
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
}

impl Encode for Axm {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi
            .encode(buf)
            .map_err(|e| e.nested().context("Axm::reqi"))?;
        let len = self.info.len();
        if len > AXM_MAX_OBJECTS {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: AXM_MAX_OBJECTS,
                found: len,
            }
            .context("Axm: Too many AXM objects"));
        }
        (len as u8)
            .encode(buf)
            .map_err(|e| e.nested().context("Axm::len"))?;
        self.ucid
            .encode(buf)
            .map_err(|e| e.nested().context("Axm::ucid"))?;
        self.pmoaction
            .encode(buf)
            .map_err(|e| e.nested().context("Axm::pmoaction"))?;
        self.pmoflags
            .encode(buf)
            .map_err(|e| e.nested().context("Axm::pmoflags"))?;
        buf.put_bytes(0, 1);
        for i in self.info.iter() {
            i.encode(buf).map_err(|e| e.nested().context("Axm::info"))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use insim_core::object::control::{Control, ControlKind};

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
                0,   // info[1] - objectindex
                128, // info[1] - heading
                172, // info[2] - x (1)
                218, // info[2] - x (2)
                25,  // info[2] - y (1)
                136, // info[2] - y (2)
                8,   // info[2] - zbyte
                0,   // info[2] - flags
                0,   // info[2] - objectindex
                128, // info[2] - heading
            ],
            |axm: Axm| {
                // xyz: Coordinate {
                //     x:
                //     y: -1918.4375, // -30695.0 / 16.0
                //     z: 2.0 // 8.0 / 4
                // },

                assert_eq!(axm.info.len(), 2);
                assert_eq!(
                    axm.info[0].position().x_metres(),
                    -597.25 /* -9556.0 / 16.0 */
                );
                assert_eq!(
                    axm.info[0].position().y_metres(),
                    -1918.4375 /* -30695.0 / 16.0 */
                );
                assert_eq!(axm.info[0].position().z_metres(), 2.0 /* 8.0 / 4 */);
                assert!(matches!(
                    axm.info[0],
                    ObjectInfo::Control(Control {
                        kind: ControlKind::Start,
                        floating: false,
                        ..
                    })
                ));
            }
        )
    }
}
