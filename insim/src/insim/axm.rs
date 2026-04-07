use insim_core::{Decode, DecodeContext, Encode, EncodeContext, object::ObjectCoordinate};

use crate::identifiers::{ConnectionId, RequestId};

const AXM_MAX_OBJECTS: usize = 60;

pub use insim_core::object::ObjectInfo;

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
enum PmoActionWire {
    #[default]
    LoadingFile = 0,
    AddObjects = 1,
    DelObjects = 2,
    ClearAll = 3,
    TinyAxm = 4,
    TtcSel = 5,
    Selection = 6,
    Position = 7,
    GetZ = 8,
}

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    /// Flags for an [`Axm`] packet.
    pub struct PmoFlags: u8 {
        /// LFS has reached the end of a layout file, or (on [`PmoAction::AddObjects`])
        /// requests client-side optimisation of all objects.
        const FILE_END = (1 << 0);

        /// This packet is one of a paired [`PmoAction::DelObjects`] /
        /// [`PmoAction::AddObjects`] emitted when objects are moved or modified.
        const MOVE_MODIFY = (1 << 1);

        /// On [`PmoAction::Selection`]: real object selection (CTRL+click) rather
        /// than clipboard selection (CTRL+C).
        const SELECTION_REAL = (1 << 2);

        /// On [`PmoAction::AddObjects`]: skip intersection / position validity checks
        /// on guest computers.
        const AVOID_CHECK = (1 << 3);
    }
}

impl_bitflags_from_to_bytes!(PmoFlags, u8);

/// An entry in an [`PmoAction::GetZ`] request or response.
///
/// Send entries with Zbyte set to 240 to get the highest point at X, Y,
/// or use the approximate altitude. In the reply, Zbyte is adjusted and
/// `adjusted` indicates whether the adjustment succeeded.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GetZEntry {
    /// Position (X, Y, input or output Z).
    pub xyz: ObjectCoordinate,
    /// True if Zbyte was successfully adjusted (reply only).
    pub adjusted: bool,
}

/// The action carried by an [`Axm`] packet.
///
/// Action-specific flags (e.g. [`PmoFlags::SELECTION_REAL`] for [`Selection`](PmoAction::Selection))
/// live on [`Axm::flags`] rather than inside the variant.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum PmoAction {
    /// Sent by the layout loading system while loading a file.
    LoadingFile(Vec<ObjectInfo>),

    /// Add objects to the layout.
    AddObjects(Vec<ObjectInfo>),

    /// Delete objects from the layout.
    DelObjects(Vec<ObjectInfo>),

    /// Remove all objects from the layout.
    ClearAll,

    /// Reply to [`TinyType::Axm`](crate::insim::TinyType::Axm).
    TinyAxm(Vec<ObjectInfo>),

    /// Reply to [`TtcType::Sel`](crate::insim::TtcType::Sel).
    TtcSel(Vec<ObjectInfo>),

    /// Set or report a connection's layout editor selection.
    Selection(Vec<ObjectInfo>),

    /// User pressed 'O' without anything selected; reports current editor position.
    ///
    /// Information only — no object is added. Only `xyz` and `heading` are meaningful.
    Position {
        /// Position of the editor cursor.
        xyz: ObjectCoordinate,
        /// Raw heading byte.
        heading: u8,
    },

    /// Request or reply with Z values for given X, Y positions.
    ///
    /// Send entries with suggested Zbyte values; receive them back with adjusted
    /// Zbyte values and [`GetZEntry::adjusted`] set to indicate success.
    GetZ(Vec<GetZEntry>),
}

impl Default for PmoAction {
    fn default() -> Self {
        Self::LoadingFile(Vec::new())
    }
}

/// AutoX multiple objects update.
///
/// Adds, removes, or reports layout objects. Action-specific flags are on
/// [`flags`](Axm::flags); the action and its data are on [`action`](Axm::action).
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Axm {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Connection that sent or requested the update.
    pub ucid: ConnectionId,

    /// Flags qualifying the action.
    pub flags: PmoFlags,

    /// The action and its associated objects or data.
    pub action: PmoAction,
}

impl_typical_with_request_id!(Axm);

impl Decode for Axm {
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let reqi = ctx.decode::<RequestId>("reqi")?;
        let numo = ctx.decode::<u8>("numo")?;
        let ucid = ctx.decode::<ConnectionId>("ucid")?;
        let pmoaction = ctx.decode::<PmoActionWire>("pmoaction")?;
        let flags = ctx.decode::<PmoFlags>("pmoflags")?;
        ctx.pad("sp3", 1)?;

        let decode_objects =
            |ctx: &mut DecodeContext, n: u8| -> Result<Vec<ObjectInfo>, insim_core::DecodeError> {
                (0..n).map(|_| ctx.decode::<ObjectInfo>("info")).collect()
            };

        let action = match pmoaction {
            PmoActionWire::LoadingFile => PmoAction::LoadingFile(decode_objects(ctx, numo)?),
            PmoActionWire::AddObjects => PmoAction::AddObjects(decode_objects(ctx, numo)?),
            PmoActionWire::DelObjects => PmoAction::DelObjects(decode_objects(ctx, numo)?),
            PmoActionWire::ClearAll => PmoAction::ClearAll,
            PmoActionWire::TinyAxm => PmoAction::TinyAxm(decode_objects(ctx, numo)?),
            PmoActionWire::TtcSel => PmoAction::TtcSel(decode_objects(ctx, numo)?),
            PmoActionWire::Selection => PmoAction::Selection(decode_objects(ctx, numo)?),
            PmoActionWire::Position => {
                let x = ctx.decode::<i16>("x")?;
                let y = ctx.decode::<i16>("y")?;
                let z = ctx.decode::<u8>("z")?;
                ctx.pad("flags", 1)?;
                ctx.pad("index", 1)?;
                let heading = ctx.decode::<u8>("heading")?;
                PmoAction::Position {
                    xyz: ObjectCoordinate::new(x, y, z),
                    heading,
                }
            },
            PmoActionWire::GetZ => {
                let entries = (0..numo)
                    .map(|_| {
                        let x = ctx.decode::<i16>("x")?;
                        let y = ctx.decode::<i16>("y")?;
                        let z = ctx.decode::<u8>("z")?;
                        let flags = ctx.decode::<u8>("flags")?;
                        ctx.pad("index", 1)?;
                        ctx.pad("heading", 1)?;
                        Ok(GetZEntry {
                            xyz: ObjectCoordinate::new(x, y, z),
                            adjusted: flags & 0x80 != 0,
                        })
                    })
                    .collect::<Result<Vec<_>, insim_core::DecodeError>>()?;
                PmoAction::GetZ(entries)
            },
        };

        Ok(Self {
            reqi,
            ucid,
            flags,
            action,
        })
    }
}

impl Encode for Axm {
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        let (pmoaction, numo): (PmoActionWire, usize) = match &self.action {
            PmoAction::LoadingFile(info) => (PmoActionWire::LoadingFile, info.len()),
            PmoAction::AddObjects(info) => (PmoActionWire::AddObjects, info.len()),
            PmoAction::DelObjects(info) => (PmoActionWire::DelObjects, info.len()),
            PmoAction::ClearAll => (PmoActionWire::ClearAll, 0),
            PmoAction::TinyAxm(info) => (PmoActionWire::TinyAxm, info.len()),
            PmoAction::TtcSel(info) => (PmoActionWire::TtcSel, info.len()),
            PmoAction::Selection(info) => (PmoActionWire::Selection, info.len()),
            PmoAction::Position { .. } => (PmoActionWire::Position, 1),
            PmoAction::GetZ(entries) => (PmoActionWire::GetZ, entries.len()),
        };

        if numo > AXM_MAX_OBJECTS {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: AXM_MAX_OBJECTS,
                found: numo,
            }
            .context("Axm: Too many AXM objects"));
        }

        ctx.encode("reqi", &self.reqi)?;
        ctx.encode("numo", &(numo as u8))?;
        ctx.encode("ucid", &self.ucid)?;
        ctx.encode("pmoaction", &pmoaction)?;
        ctx.encode("pmoflags", &self.flags)?;
        ctx.pad("sp3", 1)?;

        match &self.action {
            PmoAction::LoadingFile(info)
            | PmoAction::AddObjects(info)
            | PmoAction::DelObjects(info)
            | PmoAction::TinyAxm(info)
            | PmoAction::TtcSel(info)
            | PmoAction::Selection(info) => {
                for obj in info {
                    ctx.encode("info", obj)?;
                }
            },
            PmoAction::GetZ(entries) => {
                for entry in entries {
                    ctx.encode("x", &entry.xyz.x)?;
                    ctx.encode("y", &entry.xyz.y)?;
                    ctx.encode("z", &entry.xyz.z)?;
                    ctx.encode("flags", &(if entry.adjusted { 0x80u8 } else { 0u8 }))?;
                    ctx.pad("index", 1)?;
                    ctx.pad("heading", 1)?;
                }
            },
            PmoAction::Position { xyz, heading } => {
                ctx.encode("x", &xyz.x)?;
                ctx.encode("y", &xyz.y)?;
                ctx.encode("z", &xyz.z)?;
                ctx.pad("flags", 1)?;
                ctx.pad("index", 1)?;
                ctx.encode("heading", heading)?;
            },
            PmoAction::ClearAll => {},
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
                1,   // pmoaction (AddObjects)
                8,   // pmoflags (AVOID_CHECK = 1 << 3)
                0,   // sp3
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
                assert!(axm.flags.contains(PmoFlags::AVOID_CHECK));
                assert!(!axm.flags.contains(PmoFlags::MOVE_MODIFY));

                let PmoAction::AddObjects(info) = axm.action else {
                    panic!("expected AddObjects action");
                };

                assert_eq!(info.len(), 2);
                assert_eq!(info[0].position().x_metres(), -597.25);
                assert_eq!(info[0].position().y_metres(), -1918.4375);
                assert_eq!(info[0].position().z_metres(), 2.0);
                assert!(matches!(
                    info[0],
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
