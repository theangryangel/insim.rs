//! Strongly typed Vehicles for both standard and mods
use crate::{license::License, Error, ReadWriteBuf};

/// Handles parsing a vehicle name according to the Insim v9 rules.
/// See <https://www.lfs.net/forum/thread/95662-New-InSim-packet-size-byte-and-mod-info>
#[derive(PartialEq, Eq, Clone, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum Vehicle {
    #[default]
    Xfg,
    Xrg,
    Fbm,

    Xrt,
    Rb4,
    Fxo,
    Lx4,
    Lx6,
    Mrt,

    Uf1,
    Rac,
    Fz5,
    Fox,
    Xfr,
    Ufr,
    Fo8,
    Fxr,
    Xrr,
    Fzr,
    Bf1,

    Mod(u32),

    /// Unknown vehicle. *Probably* a private mod?
    Unknown,
}

#[allow(missing_docs)]
impl Vehicle {
    pub fn is_builtin(&self) -> bool {
        !matches!(self, Vehicle::Mod(_))
    }

    pub fn is_mod(&self) -> bool {
        matches!(self, Vehicle::Mod(_))
    }

    pub fn license(&self) -> License {
        match self {
            Vehicle::Xfg => License::Demo,
            Vehicle::Xrg => License::Demo,
            Vehicle::Fbm => License::Demo,

            Vehicle::Xrt => License::S1,
            Vehicle::Rb4 => License::S1,
            Vehicle::Fxo => License::S1,
            Vehicle::Lx4 => License::S1,
            Vehicle::Lx6 => License::S1,
            Vehicle::Mrt => License::S1,

            Vehicle::Uf1 => License::S2,
            Vehicle::Rac => License::S2,
            Vehicle::Fz5 => License::S2,
            Vehicle::Fox => License::S2,
            Vehicle::Xfr => License::S2,
            Vehicle::Ufr => License::S2,
            Vehicle::Fo8 => License::S2,
            Vehicle::Fxr => License::S2,
            Vehicle::Xrr => License::S2,
            Vehicle::Fzr => License::S2,
            Vehicle::Bf1 => License::S2,
            Vehicle::Mod(_) => License::S3,
            Vehicle::Unknown => License::S3,
        }
    }
}

impl ReadWriteBuf for Vehicle {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, crate::Error> {
        let mut bytes = buf.split_to(4);
        let is_builtin = bytes[0..=2].iter().all(|c| c.is_ascii_alphanumeric()) && bytes[3] == 0;

        match (bytes.as_ref(), is_builtin) {
            ([0, 0, 0, 0], _) => Ok(Vehicle::Unknown),
            ([b'X', b'F', b'G', 0], true) => Ok(Vehicle::Xfg),
            ([b'X', b'R', b'G', 0], true) => Ok(Vehicle::Xrg),
            ([b'F', b'B', b'M', 0], true) => Ok(Vehicle::Fbm),
            ([b'X', b'R', b'T', 0], true) => Ok(Vehicle::Xrt),
            ([b'R', b'B', b'4', 0], true) => Ok(Vehicle::Rb4),
            ([b'F', b'X', b'O', 0], true) => Ok(Vehicle::Fxo),
            ([b'L', b'X', b'4', 0], true) => Ok(Vehicle::Lx4),
            ([b'L', b'X', b'6', 0], true) => Ok(Vehicle::Lx6),
            ([b'M', b'R', b'T', 0], true) => Ok(Vehicle::Mrt),
            ([b'U', b'F', b'1', 0], true) => Ok(Vehicle::Uf1),
            ([b'R', b'A', b'C', 0], true) => Ok(Vehicle::Rac),
            ([b'F', b'Z', b'5', 0], true) => Ok(Vehicle::Fz5),
            ([b'F', b'O', b'X', 0], true) => Ok(Vehicle::Fox),
            ([b'X', b'F', b'R', 0], true) => Ok(Vehicle::Xfr),
            ([b'U', b'F', b'R', 0], true) => Ok(Vehicle::Ufr),
            ([b'F', b'O', b'8', 0], true) => Ok(Vehicle::Fo8),
            ([b'F', b'X', b'R', 0], true) => Ok(Vehicle::Fxr),
            ([b'X', b'R', b'R', 0], true) => Ok(Vehicle::Xrr),
            ([b'F', b'Z', b'R', 0], true) => Ok(Vehicle::Fzr),
            ([b'B', b'F', b'1', 0], true) => Ok(Vehicle::Bf1),
            (_, true) => Err(Error::BadMagic {
                found: Box::new(bytes),
            }),
            (_, false) => Ok(Vehicle::Mod(u32::read_buf(&mut bytes)?)),
        }
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::Error> {
        match self {
            Vehicle::Xfg => buf.extend_from_slice(&[b'X', b'F', b'G', 0]),
            Vehicle::Xrg => buf.extend_from_slice(&[b'X', b'R', b'G', 0]),
            Vehicle::Fbm => buf.extend_from_slice(&[b'F', b'B', b'M', 0]),
            Vehicle::Xrt => buf.extend_from_slice(&[b'X', b'R', b'T', 0]),
            Vehicle::Rb4 => buf.extend_from_slice(&[b'R', b'B', b'4', 0]),
            Vehicle::Fxo => buf.extend_from_slice(&[b'F', b'X', b'O', 0]),
            Vehicle::Lx4 => buf.extend_from_slice(&[b'L', b'X', b'4', 0]),
            Vehicle::Lx6 => buf.extend_from_slice(&[b'L', b'X', b'6', 0]),
            Vehicle::Mrt => buf.extend_from_slice(&[b'M', b'R', b'T', 0]),
            Vehicle::Uf1 => buf.extend_from_slice(&[b'U', b'F', b'1', 0]),
            Vehicle::Rac => buf.extend_from_slice(&[b'R', b'A', b'C', 0]),
            Vehicle::Fz5 => buf.extend_from_slice(&[b'F', b'Z', b'5', 0]),
            Vehicle::Fox => buf.extend_from_slice(&[b'F', b'O', b'X', 0]),
            Vehicle::Xfr => buf.extend_from_slice(&[b'X', b'F', b'R', 0]),
            Vehicle::Ufr => buf.extend_from_slice(&[b'U', b'F', b'R', 0]),
            Vehicle::Fo8 => buf.extend_from_slice(&[b'F', b'O', b'8', 0]),
            Vehicle::Fxr => buf.extend_from_slice(&[b'F', b'X', b'R', 0]),
            Vehicle::Xrr => buf.extend_from_slice(&[b'X', b'R', b'R', 0]),
            Vehicle::Fzr => buf.extend_from_slice(&[b'F', b'Z', b'R', 0]),
            Vehicle::Bf1 => buf.extend_from_slice(&[b'B', b'F', b'1', 0]),
            Vehicle::Mod(vehmod) => vehmod.write_buf(buf)?,
            Vehicle::Unknown => buf.extend_from_slice(&[0_u8, 0_u8, 0_u8, 0_u8]),
        };

        Ok(())
    }
}

impl std::fmt::Display for Vehicle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Vehicle::Xfg => write!(f, "XFG"),
            Vehicle::Xrg => write!(f, "XRG"),
            Vehicle::Fbm => write!(f, "FBM"),
            Vehicle::Xrt => write!(f, "XRT"),
            Vehicle::Rb4 => write!(f, "RB4"),
            Vehicle::Fxo => write!(f, "FXO"),
            Vehicle::Lx4 => write!(f, "LX4"),
            Vehicle::Lx6 => write!(f, "LX6"),
            Vehicle::Mrt => write!(f, "MRT"),
            Vehicle::Uf1 => write!(f, "UF1"),
            Vehicle::Rac => write!(f, "RAC"),
            Vehicle::Fz5 => write!(f, "FZ5"),
            Vehicle::Fox => write!(f, "FOX"),
            Vehicle::Xfr => write!(f, "XFR"),
            Vehicle::Ufr => write!(f, "UFR"),
            Vehicle::Fo8 => write!(f, "FO8"),
            Vehicle::Fxr => write!(f, "FXR"),
            Vehicle::Xrr => write!(f, "XRR"),
            Vehicle::Fzr => write!(f, "FZR"),
            Vehicle::Bf1 => write!(f, "BF1"),
            Vehicle::Mod(vehmod) => {
                // Determine the mod id. This is only applicable for Insim v9.
                write!(f, "{:06X}", vehmod)
            },
            Vehicle::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Debug for Vehicle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Vehicle::Xfg => write!(f, "XFG"),
            Vehicle::Xrg => write!(f, "XRG"),
            Vehicle::Fbm => write!(f, "FBM"),
            Vehicle::Xrt => write!(f, "XRT"),
            Vehicle::Rb4 => write!(f, "RB4"),
            Vehicle::Fxo => write!(f, "FXO"),
            Vehicle::Lx4 => write!(f, "LX4"),
            Vehicle::Lx6 => write!(f, "LX6"),
            Vehicle::Mrt => write!(f, "MRT"),
            Vehicle::Uf1 => write!(f, "UF1"),
            Vehicle::Rac => write!(f, "RAC"),
            Vehicle::Fz5 => write!(f, "FZ5"),
            Vehicle::Fox => write!(f, "FOX"),
            Vehicle::Xfr => write!(f, "XFR"),
            Vehicle::Ufr => write!(f, "UFR"),
            Vehicle::Fo8 => write!(f, "FO8"),
            Vehicle::Fxr => write!(f, "FXR"),
            Vehicle::Xrr => write!(f, "XRR"),
            Vehicle::Fzr => write!(f, "FZR"),
            Vehicle::Bf1 => write!(f, "BF1"),
            Vehicle::Mod(vehmod) => {
                // Determine the mod id. This is only applicable for Insim v9.
                write!(f, "MOD({:06X})", vehmod)
            },
            Vehicle::Unknown => write!(f, "Unknown"),
        }
    }
}
