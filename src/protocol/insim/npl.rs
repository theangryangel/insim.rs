use crate::packet_flags;
use crate::string::{is_ascii_alphanumeric, strip_trailing_nul, ICodepageString, IString};
use deku::prelude::*;
use serde::Serialize;

/// Handles parsing a vehicle name according to the Insim v9 rules.
/// See https://www.lfs.net/forum/thread/95662-New-InSim-packet-size-byte-and-mod-info
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
pub struct VehicleName {
    pub inner: [u8; 4],
}

impl VehicleName {
    /// Is this a built in vehicle?
    pub fn is_builtin(&self) -> bool {
        is_ascii_alphanumeric(&self.inner[0])
            && is_ascii_alphanumeric(&self.inner[1])
            && is_ascii_alphanumeric(&self.inner[2])
            && self.inner[3] == 0
    }

    /// Is this a mod? This is only applicable for >= Insim v9
    pub fn is_mod(&self) -> bool {
        !self.is_builtin()
    }

    /// Determine the mod id. This is only applicable for Insim v9
    pub fn mod_id(&self) -> String {
        let value = u32::from_le_bytes(self.inner);
        format!("{:06X}", value)
    }
}

impl std::fmt::Display for VehicleName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_builtin() {
            let stripped = strip_trailing_nul(&self.inner);
            write!(f, "{}", String::from_utf8_lossy(stripped))
        } else {
            write!(f, "{}", self.mod_id())
        }
    }
}

packet_flags! {
    #[derive(Serialize)]
    pub struct PlayerFlags: u16 {
        SWAPSIDE => (1 << 0),
        RESERVED_2 => (1 << 1),
        RESERVED_4 => (1 << 2),
        AUTOGEARS => (1 << 3),
        SHIFTER => (1 << 4),
        RESERVED_32 => (1 << 5),
        HELP_B => (1 << 6),
        AXIS_CLUTCH => (1 << 7),
        INPITS => (1 << 8),
        AUTOCLUTCH => (1 << 9),
        MOUSE => (1 << 10),
        KB_NO_HELP => (1 << 11),
        KB_STABILISED => (1 << 12),
        CUSTOM_VIEW => (1 << 13),
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Sent when a New Player joins.
pub struct Npl {
    pub reqi: u8,

    pub plid: u8,

    pub ucid: u8,

    pub ptype: u8,

    pub flags: PlayerFlags,

    #[deku(bytes = "24")]
    pub pname: ICodepageString,

    #[deku(bytes = "8")]
    pub plate: ICodepageString,

    pub cname: VehicleName,

    #[deku(bytes = "16")]
    pub sname: IString,

    #[deku(bytes = "1", count = "4")]
    pub tyres: Vec<u8>,

    pub h_mass: u8,

    pub h_tres: u8,

    pub model: u8,

    pub pass: u8,

    pub rwadj: u8,

    #[deku(pad_bytes_after = "2")]
    pub fwadj: u8,

    pub setf: u8,

    pub nump: u8,

    pub config: u8,

    pub fuel: u8,
}
