use bitflags::bitflags;
use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
    vehicle::Vehicle,
};

use super::Fuel;
use crate::identifiers::{ConnectionId, PlayerId, RequestId};

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
#[non_exhaustive]
/// Tyre compounds/types
pub enum TyreCompound {
    /// R1
    R1 = 0,

    /// R2
    R2 = 1,

    /// R3
    R3 = 2,

    /// R4
    R4 = 3,

    /// Road super
    RoadSuper = 4,

    /// Road normal
    RoadNormal = 5,

    /// Hybrid
    Hybrid = 6,

    /// Knobbly/Off-road
    Knobbly = 7,

    /// Special: "No change"
    #[default]
    NoChange = 255,
}

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    /// Describes the setup of a player and the various helpers that may be enabled, such as
    /// auto-clutch, etc.
    pub struct PlayerFlags: u16 {
        /// Left side
        const LEFTSIDE = (1 << 0);
        // const RESERVED_2 = (1 << 1);
        // const RESERVED_4 = (1 << 2);
        /// Autogears
        const AUTOGEARS = (1 << 3);
        /// Shifter
        const SHIFTER = (1 << 4);
        // const RESERVED_32 = (1 << 5);
        /// "Help_B"
        const HELP_B = (1 << 6);
        /// Axis clutch
        const AXIS_CLUTCH = (1 << 7);
        /// In pits
        const INPITS = (1 << 8);
        /// Autoclutch
        const AUTOCLUTCH = (1 << 9);
        /// Mouse
        const MOUSE = (1 << 10);
        /// Keyboard, without assistance/help
        const KB_NO_HELP = (1 << 11);
        /// Key, with assistance/help
        const KB_STABILISED = (1 << 12);
        /// Custom view
        const CUSTOM_VIEW = (1 << 13);
    }
}

generate_bitflag_helpers!(PlayerFlags,
    pub is_left_side => LEFTSIDE,
    pub using_auto_gear_shift => AUTOGEARS,
    pub has_shifter => SHIFTER,
    pub in_pits => INPITS,
    pub using_auto_clutch => AUTOCLUTCH,
    pub using_mouse => MOUSE,
    pub using_keyboard => KB_NO_HELP,
    pub using_keyboard_with_stabilisation => KB_STABILISED,
    pub using_custom_view => CUSTOM_VIEW
);

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    /// Setup Flags
    pub struct SetFlags: u8 {
        /// Symmetric wheels
        const SYMM_WHEELS = (1 << 0);
        /// Traction Control enabled
        const TC_ENABLE = (1 << 1);
        /// ABS (Anti-lock Braking System) enabled
        const ABS_ENABLE = (1 << 2);
    }
}

generate_bitflag_helpers!(SetFlags,
    pub is_symmetric => SYMM_WHEELS,
    pub is_traction_control_enabled => TC_ENABLE,
    pub is_anti_lock_braking_enabled => ABS_ENABLE
);

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    /// Player model and type information
    pub struct PlayerType: u8 {
        /// Female, if not set assume male
        const FEMALE = (1 << 0);
        /// AI
        const AI = (1 << 1);
        /// Remote
        const REMOTE = (1 << 2);
    }
}

generate_bitflag_helpers!(
    PlayerType,
    pub is_female => FEMALE,
    pub is_ai => AI,
    pub is_remote => REMOTE
);

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    /// Passenger flags
    pub struct Passengers: u8 {
        /// Front male, opposite side from driver
        const FRONT_MALE = (1 << 0);
        /// Front female, opposite side from driver
        const FRONT_FEMALE = (1 << 1);
        /// Rear left, male
        const REAR_LEFT_MALE = (1 << 2);
        /// Rear left, female
        const REAR_LEFT_FEMALE = (1 << 3);
        /// Rear middle, male
        const REAR_MIDDLE_MALE = (1 << 4);
        /// Rear middle, female
        const REAR_MIDDLE_FEMALE = (1 << 5);
        /// Rear right, male
        const REAR_RIGHT_MALE = (1 << 6);
        /// Rear right, female
        const REAR_RIGHT_FEMALE = (1 << 7);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Sent when a New Player joins.
pub struct Npl {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player id given to this new player
    pub plid: PlayerId,

    /// Unique connection id of this player
    pub ucid: ConnectionId,

    /// See [PlayerType].
    pub ptype: PlayerType,

    /// See [PlayerFlags].
    pub flags: PlayerFlags,

    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    /// Player name
    pub pname: String,

    #[bw(write_with = binrw_write_codepage_string::<8, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<8, _>)]
    /// Number plate
    pub plate: String,

    /// Vehicle they've joined with.
    pub cname: Vehicle,

    #[bw(write_with = binrw_write_codepage_string::<16, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<16, _>)]
    /// Skin name.
    pub sname: String,

    /// TyreCompound for each tyre.
    pub tyres: [TyreCompound; 4],

    /// added mass (kg)
    pub h_mass: u8,
    /// intake restriction
    pub h_tres: u8,

    /// Driver model
    pub model: u8,

    /// Passengers
    pub pass: Passengers,

    /// low 4 bits: tyre width reduction (rear)
    pub rwadj: u8, // TODO: split into pair of u4

    /// low 4 bits: tyre width reduction (front)
    #[brw(pad_after = 2)]
    pub fwadj: u8, // TODO: split into pair of u4

    /// Setup flags, see [SetFlags].
    pub setf: SetFlags,

    /// Total number of players in server
    pub nump: u8,

    /// Configuration.
    /// UF1 / LX4 / LX6: 0 = DEFAULT / 1 = OPEN ROOF
    /// GTR racing cars: 0 = DEFAULT / 1 = ALTERNATE
    pub config: u8,

    /// When /showfuel yes: fuel percent / no: 255
    pub fuel: Fuel,
}
