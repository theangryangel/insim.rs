use insim_core::{Decode, Encode};

use crate::identifiers::{ConnectionId, RequestId};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
/// Interface mode reported by [Cim].
pub enum CimMode {
    /// Not in a special mode
    Normal(CimSubModeNormal),

    /// Options screen
    Options,

    /// Host options screen
    HostOptions,

    /// Garage screen
    Garage(CimSubModeGarage),

    /// Vehicle select screen
    CarSelect,

    /// Track select screen
    TrackSelect,

    /// Shift+U mode
    ShiftU {
        /// ShiftU submode
        submode: CimSubModeShiftU,

        /// Selected object type (0 if none).
        /// Special values are used for InSim checkpoints, circles, marshal zones, and routes.
        seltype: u8,
    },
}

impl Default for CimMode {
    fn default() -> Self {
        Self::Normal(CimSubModeNormal::Normal)
    }
}

impl Decode for CimMode {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let discrim = u8::decode(buf)?;
        let submode = u8::decode(buf)?;
        let seltype = u8::decode(buf)?;

        let res = match discrim {
            0 => Self::Normal(submode.into()),
            1 => Self::Options,
            2 => Self::HostOptions,
            3 => Self::Garage(submode.into()),
            4 => Self::CarSelect,
            5 => Self::TrackSelect,
            6 => Self::ShiftU {
                submode: submode.into(),
                seltype,
            },
            found => {
                return Err(insim_core::DecodeErrorKind::NoVariantMatch {
                    found: found as u64,
                }
                .into());
            },
        };

        Ok(res)
    }
}

impl Encode for CimMode {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        let (discrim, submode, seltype) = match self {
            CimMode::Normal(submode) => (0u8, *submode as u8, 0u8),
            CimMode::Options => (1u8, 0u8, 0u8),
            CimMode::HostOptions => (2u8, 0u8, 0u8),
            CimMode::Garage(submode) => (3u8, *submode as u8, 0u8),
            CimMode::CarSelect => (4u8, 0u8, 0u8),
            CimMode::TrackSelect => (5u8, 0u8, 0u8),
            CimMode::ShiftU {
                submode: mode,
                seltype,
            } => (6u8, *mode as u8, *seltype),
        };

        discrim.encode(buf)?;
        submode.encode(buf)?;
        seltype.encode(buf)?;
        Ok(())
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
/// Submode for `CimMode::Normal`.
pub enum CimSubModeNormal {
    #[default]
    /// Not in a special mode
    Normal = 0,

    /// Showing wheel temperature
    WheelTemps = 1,

    /// Showing wheel damaage
    WheelDamage = 2,

    /// Showing live settings
    LiveSettings = 3,

    /// Show pit instructions
    PitInstructions = 4,
}

impl From<u8> for CimSubModeNormal {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::WheelTemps,
            2 => Self::WheelDamage,
            3 => Self::LiveSettings,
            4 => Self::PitInstructions,
            other => {
                unreachable!(
                    "Unhandled CimSubModeNormal. Perhaps a programming error or protocol update? Found {}, expected 0-4.",
                    other
                )
            },
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
/// Submode for `CimMode::Garage`.
pub enum CimSubModeGarage {
    #[default]
    /// Info tab of setup screen
    Info = 0,

    /// Colours tab of setup screen
    Colours = 1,

    /// Braking and traction control tab of setup screen
    BrakeTC = 2,

    /// Suspension tab of setup screen
    Susp = 3,

    /// Steering tab of setup screen
    Steer = 4,

    /// Drive / gear tab of setup screen
    Drive = 5,

    /// Tyres
    Tyres = 6,

    /// Aero tab of setup screen
    Aero = 7,

    /// Passengers tab of setup screen
    Pass = 8,

    /// Undocumented mod review tab
    ModReview = 255,
}

impl From<u8> for CimSubModeGarage {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Info,
            1 => Self::Colours,
            2 => Self::BrakeTC,
            3 => Self::Susp,
            4 => Self::Steer,
            5 => Self::Drive,
            6 => Self::Tyres,
            7 => Self::Aero,
            8 => Self::Pass,
            255 => Self::ModReview,
            other => {
                unreachable!(
                    "Unhandled CimSubModeGarage. Perhaps a programming error or protocol update? Found {}, expected 0-8",
                    other
                )
            },
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
/// Submode for `CimMode::ShiftU`.
pub enum CimSubModeShiftU {
    #[default]
    /// No buttons displayed
    Plain = 0,

    /// Buttons displayed, but not editing
    Buttons = 1,

    /// Editing mode
    Edit = 2,
}

impl From<u8> for CimSubModeShiftU {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Buttons,
            2 => Self::Edit,
            _ => Self::Plain,
        }
    }
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Connection interface mode report.
///
/// - Indicates which UI screen/mode a connection is in.
pub struct Cim {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Connection identifier (0 = local).
    pub ucid: ConnectionId,

    /// Mode and submode information.
    #[insim(pad_after = 1)]
    pub mode: CimMode,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cim() {
        assert_from_to_bytes!(
            Cim,
            [
                0, // reqi
                4, // ucid
                6, // mode
                2, // submode
                5, // seltype
                0, // sp3
            ],
            |cim: Cim| {
                assert_eq!(cim.reqi, RequestId(0));
                assert_eq!(cim.ucid, ConnectionId(4));
                assert!(matches!(
                    cim.mode,
                    CimMode::ShiftU {
                        submode: CimSubModeShiftU::Edit,
                        seltype: 5
                    },
                ));
            }
        );
    }
}
