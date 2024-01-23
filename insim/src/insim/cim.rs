use insim_core::binrw::{self, binrw, BinRead, BinWrite};

use crate::identifiers::{ConnectionId, RequestId};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Used within the [Cim] packet to indicate the mode.
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
    VehicleSelect,

    /// Track select screen
    TrackSelect,

    /// Shift+U mode
    ShiftU {
        /// ShiftU submode
        submode: CimSubModeShiftU,

        /// SelType is the selected object type or zero if unselected
        /// It may be an AXO_x as in ObjectInfo or one of these:
        /// const int MARSH_IS_CP = 252; // insim checkpoint
        /// const int MARSH_IS_AREA = 253; // insim circle
        /// const int MARSH_MARSHAL = 254; // restricted area
        /// const int MARSH_ROUTE = 255; // route checker
        seltype: u8,
    },
}

impl Default for CimMode {
    fn default() -> Self {
        Self::Normal(CimSubModeNormal::Normal)
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
/// CimMode::Normal, submode
pub enum CimSubModeNormal {
    #[default]
    /// Not in a special mode
    Normal = 0,

    /// Showing wheel temperature
    WheelTemps = 1,

    /// Showing wheel damaage
    WheelDamage = 2,

    /// Showing live setings
    LiveSettings = 3,

    /// Show pit instructions
    PitInstructions = 4,
}

impl From<u8> for CimSubModeNormal {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::WheelTemps,
            2 => Self::WheelDamage,
            3 => Self::LiveSettings,
            4 => Self::LiveSettings,
            _ => Self::Normal,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
/// CimMode::Garage, submode
pub enum CimSubModeGarage {
    #[default]
    /// Info tab of setup screen
    Info = 0,

    /// Colours tab of setup screen
    Colours = 1,

    /// Braking and traction control tab of setup screen
    BrakeTC = 2,

    /// Suspension tab of setup screen
    Suspension = 3,

    /// Steering tab of setup screen
    Steer = 4,

    /// Drive / gear tab of setup screen
    Drive = 5,

    /// Aero tab of setup screen
    Aero = 6,

    /// Passengers tab of setup screen
    Passengers = 7,
}

impl From<u8> for CimSubModeGarage {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Colours,
            2 => Self::BrakeTC,
            3 => Self::Suspension,
            4 => Self::Steer,
            5 => Self::Drive,
            6 => Self::Aero,
            7 => Self::Passengers,
            _ => Self::Info,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
/// CimMode::ShiftU, submode
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

impl BinRead for CimMode {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let pos = reader.stream_position()?;
        let discrim = u8::read_options(reader, endian, ())?;
        let submode = u8::read_options(reader, endian, ())?;
        let seltype = u8::read_options(reader, endian, ())?;

        let res = match discrim {
            0 => Self::Normal(submode.into()),
            1 => Self::Options,
            2 => Self::HostOptions,
            3 => Self::Garage(submode.into()),
            4 => Self::VehicleSelect,
            5 => Self::TrackSelect,
            6 => Self::ShiftU {
                submode: submode.into(),
                seltype,
            },
            _ => {
                return Err(binrw::Error::BadMagic {
                    pos,
                    found: Box::new(submode),
                })
            }
        };

        Ok(res)
    }
}

impl BinWrite for CimMode {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        let (discrim, submode, seltype) = match self {
            CimMode::Normal(submode) => (0u8, *submode as u8, 0u8),
            CimMode::Options => (1u8, 0u8, 0u8),
            CimMode::HostOptions => (2u8, 0u8, 0u8),
            CimMode::Garage(submode) => (3u8, *submode as u8, 0u8),
            CimMode::VehicleSelect => (4u8, 0u8, 0u8),
            CimMode::TrackSelect => (5u8, 0u8, 0u8),
            CimMode::ShiftU {
                submode: mode,
                seltype,
            } => (6u8, *mode as u8, *seltype),
        };

        discrim.write_options(writer, endian, ())?;
        submode.write_options(writer, endian, ())?;
        seltype.write_options(writer, endian, ())?;
        Ok(())
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Connection Interface Mode
pub struct Cim {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// connection's unique id (0 = local)
    pub ucid: ConnectionId,

    /// Mode & submode
    #[brw(pad_after = 1)]
    pub mode: CimMode,
}
