use bitflags::bitflags;
use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
    vehicle::Vehicle,
    Error, FromToCodepageBytes, ReadWriteBuf,
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

impl ReadWriteBuf for TyreCompound {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let discrim = u8::read_buf(buf)?;
        let val = match discrim {
            0 => Self::R1,
            1 => Self::R2,
            2 => Self::R3,
            3 => Self::R4,
            4 => Self::RoadSuper,
            5 => Self::RoadNormal,
            6 => Self::Hybrid,
            7 => Self::Knobbly,
            255 => Self::NoChange,
            found => {
                return Err(Error::NoVariantMatch {
                    found: found as u64,
                })
            },
        };

        Ok(val)
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        let discrim: u8 = match self {
            Self::R1 => 0,
            Self::R2 => 1,
            Self::R3 => 2,
            Self::R4 => 3,
            Self::RoadSuper => 4,
            Self::RoadNormal => 5,
            Self::Hybrid => 6,
            Self::Knobbly => 7,
            Self::NoChange => 255,
        };

        discrim.write_buf(buf)?;
        Ok(())
    }
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

impl_bitflags_from_to_bytes!(PlayerFlags, u16);

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

impl_bitflags_from_to_bytes!(SetFlags, u8);

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

impl_bitflags_from_to_bytes!(PlayerType, u8);

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

impl_bitflags_from_to_bytes!(Passengers, u8);

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

impl ReadWriteBuf for Npl {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, Error> {
        let reqi = RequestId::read_buf(buf)?;
        let plid = PlayerId::read_buf(buf)?;
        let ucid = ConnectionId::read_buf(buf)?;
        let ptype = PlayerType::read_buf(buf)?;
        let flags = PlayerFlags::read_buf(buf)?;
        let pname = String::from_codepage_bytes(buf, 24)?;
        let plate = String::from_codepage_bytes(buf, 8)?;
        let cname = Vehicle::read_buf(buf)?;
        let sname = String::from_codepage_bytes(buf, 16)?;
        let tyres = <[TyreCompound; 4]>::read_buf(buf)?;
        let h_mass = u8::read_buf(buf)?;
        let h_tres = u8::read_buf(buf)?;
        let model = u8::read_buf(buf)?;
        let pass = Passengers::read_buf(buf)?;
        let rwadj = u8::read_buf(buf)?;
        let fwadj = u8::read_buf(buf)?;
        buf.advance(2);
        let setf = SetFlags::read_buf(buf)?;
        let nump = u8::read_buf(buf)?;
        let config = u8::read_buf(buf)?;
        let fuel = Fuel::read_buf(buf)?;
        Ok(Self {
            reqi,
            plid,
            ucid,
            ptype,
            flags,
            pname,
            plate,
            cname,
            sname,
            tyres,
            h_mass,
            h_tres,
            model,
            pass,
            rwadj,
            fwadj,
            setf,
            nump,
            config,
            fuel,
        })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), Error> {
        self.reqi.write_buf(buf)?;
        self.plid.write_buf(buf)?;
        self.ucid.write_buf(buf)?;
        self.ptype.write_buf(buf)?;
        self.flags.write_buf(buf)?;
        self.pname.to_codepage_bytes(buf, 24)?;
        self.plate.to_codepage_bytes(buf, 8)?;
        self.cname.write_buf(buf)?;
        self.sname.to_codepage_bytes(buf, 16)?;
        self.tyres.write_buf(buf)?;
        self.h_mass.write_buf(buf)?;
        self.h_tres.write_buf(buf)?;
        self.model.write_buf(buf)?;
        self.pass.write_buf(buf)?;
        self.rwadj.write_buf(buf)?;
        self.fwadj.write_buf(buf)?;
        buf.put_bytes(0, 2);
        self.setf.write_buf(buf)?;
        self.nump.write_buf(buf)?;
        self.config.write_buf(buf)?;
        self.fuel.write_buf(buf)?;
        Ok(())
    }
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
