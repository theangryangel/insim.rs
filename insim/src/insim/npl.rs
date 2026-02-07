use bitflags::bitflags;
use insim_core::vehicle::Vehicle;

use super::Fuel;
use crate::identifiers::{ConnectionId, PlayerId, RequestId};

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
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
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        /// Flexible steer
        const FLEXIBLE_STEER = (1 << 5);
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

impl_bitflags_from_to_bytes!(PlayerFlags, u16);

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl_bitflags_from_to_bytes!(SetFlags, u8);

generate_bitflag_helpers!(SetFlags,
    pub is_symmetric => SYMM_WHEELS,
    pub is_traction_control_enabled => TC_ENABLE,
    pub is_anti_lock_braking_enabled => ABS_ENABLE
);

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl_bitflags_from_to_bytes!(PlayerType, u8);

generate_bitflag_helpers!(
    PlayerType,
    pub is_female => FEMALE,
    pub is_ai => AI,
    pub is_remote => REMOTE
);

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl_bitflags_from_to_bytes!(Passengers, u8);

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Player joined race notification.
///
/// - Sent when a player joins the race (or returns from pits).
/// - Can be requested via [`TinyType::Npl`](crate::insim::TinyType::Npl).
pub struct Npl {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player identifier assigned for this race.
    pub plid: PlayerId,

    /// Connection identifier for the player.
    pub ucid: ConnectionId,

    /// Player type flags (AI/remote/female).
    pub ptype: PlayerType,

    /// Player flags (assists, controller, view settings).
    pub flags: PlayerFlags,

    #[insim(codepage(length = 24))]
    /// Player nickname.
    pub pname: String,

    #[insim(codepage(length = 8))]
    /// Number plate.
    pub plate: String,

    /// Vehicle used.
    pub cname: Vehicle,

    #[insim(codepage(length = 16))]
    /// Skin name.
    pub sname: String,

    /// Tyre compound per wheel.
    pub tyres: [TyreCompound; 4],

    /// Added mass handicap.
    pub h_mass: u8,
    /// Intake restriction handicap.
    pub h_tres: u8,

    /// Driver model identifier.
    pub model: u8,

    /// Passenger layout.
    pub pass: Passengers,

    /// Rear tyre width adjustment.
    pub rwadj: u8,

    /// Front tyre width adjustment.
    #[insim(pad_after = 2)]
    pub fwadj: u8,

    /// Setup flags.
    pub setf: SetFlags,

    /// Total number of players in the race.
    pub nump: u8,

    /// Vehicle configuration selection.
    ///
    /// - UF1 / LX4 / LX6: 0 = default, 1 = open roof.
    /// - GTR racing cars: 0 = default, 1 = alternate.
    pub config: u8,

    /// Fuel percent (if enabled).
    pub fuel: Fuel,
}

#[cfg(test)]
mod test {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_npl_xrt() {
        let mut raw = BytesMut::new();
        raw.extend_from_slice(&[
            0, // reqi
            3, // plid
            5, // ucid
            2, // ptype
            8, // flags (0)
            0, // flags (1)
        ]);

        raw.extend_from_slice("player".as_bytes());
        raw.put_bytes(0, 18);
        raw.extend_from_slice("12345678".as_bytes());
        raw.extend_from_slice(b"XRT\0");
        raw.extend_from_slice("MAX_CAR_TEX_NAME".as_bytes());
        raw.extend_from_slice(&[
            0,  // tyrerl
            1,  // tyrerr
            2,  // tyrefl
            3,  // tyrefr
            10, // h_mass
            15, // h_tres
            1,  // model
            2,  // pass
            4,  // rwadj
            5,  // fwadj
            0,  // sp2
            0,  // sp3
            4,  // setf
            20, // nump
            1,  // config
            34, // fuel
        ]);

        assert_from_to_bytes!(Npl, raw.as_ref(), |parsed: Npl| {
            assert_eq!(parsed.cname, Vehicle::Xrt);
            assert_eq!(parsed.plid, PlayerId(3));
            assert_eq!(parsed.ucid, ConnectionId(5));
            assert!(matches!(
                parsed.tyres,
                [
                    TyreCompound::R1,
                    TyreCompound::R2,
                    TyreCompound::R3,
                    TyreCompound::R4
                ]
            ))
        });
    }
}
