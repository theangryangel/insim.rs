//! Utility functions for working with vehicles and fetching vehicle data.

use crate::{license::License, string::is_ascii_alphanumeric};
use binrw::{BinRead, BinWrite};

/// Handles parsing a vehicle name according to the Insim v9 rules.
/// See <https://www.lfs.net/forum/thread/95662-New-InSim-packet-size-byte-and-mod-info>
#[derive(PartialEq, Eq, Clone, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
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

impl BinRead for Vehicle {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let pos = reader.stream_position()?;

        <[u8; 4]>::read_options(reader, endian, args).map(|bytes| {
            let is_builtin = is_ascii_alphanumeric(&bytes[0])
                && is_ascii_alphanumeric(&bytes[1])
                && is_ascii_alphanumeric(&bytes[2])
                && bytes.last() == Some(&0);

            match (bytes, is_builtin) {
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
                (_, true) => Err(binrw::Error::NoVariantMatch { pos }),
                (_, false) => Ok(Vehicle::Mod(u32::from_le_bytes(bytes))),
            }
        })?
    }
}

impl BinWrite for Vehicle {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        match self {
            Vehicle::Xfg => [b'X', b'F', b'G', 0].write_options(writer, endian, args),
            Vehicle::Xrg => [b'X', b'R', b'G', 0].write_options(writer, endian, args),
            Vehicle::Fbm => [b'F', b'B', b'M', 0].write_options(writer, endian, args),
            Vehicle::Xrt => [b'X', b'R', b'T', 0].write_options(writer, endian, args),
            Vehicle::Rb4 => [b'R', b'B', b'4', 0].write_options(writer, endian, args),
            Vehicle::Fxo => [b'F', b'X', b'O', 0].write_options(writer, endian, args),
            Vehicle::Lx4 => [b'L', b'X', b'4', 0].write_options(writer, endian, args),
            Vehicle::Lx6 => [b'L', b'X', b'6', 0].write_options(writer, endian, args),
            Vehicle::Mrt => [b'M', b'R', b'T', 0].write_options(writer, endian, args),
            Vehicle::Uf1 => [b'U', b'F', b'1', 0].write_options(writer, endian, args),
            Vehicle::Rac => [b'R', b'A', b'C', 0].write_options(writer, endian, args),
            Vehicle::Fz5 => [b'F', b'Z', b'5', 0].write_options(writer, endian, args),
            Vehicle::Fox => [b'F', b'O', b'X', 0].write_options(writer, endian, args),
            Vehicle::Xfr => [b'X', b'F', b'R', 0].write_options(writer, endian, args),
            Vehicle::Ufr => [b'U', b'F', b'R', 0].write_options(writer, endian, args),
            Vehicle::Fo8 => [b'F', b'O', b'8', 0].write_options(writer, endian, args),
            Vehicle::Fxr => [b'F', b'X', b'R', 0].write_options(writer, endian, args),
            Vehicle::Xrr => [b'X', b'R', b'R', 0].write_options(writer, endian, args),
            Vehicle::Fzr => [b'F', b'Z', b'R', 0].write_options(writer, endian, args),
            Vehicle::Bf1 => [b'B', b'F', b'1', 0].write_options(writer, endian, args),
            Vehicle::Mod(vehmod) => vehmod.write_options(writer, endian, args),
            Vehicle::Unknown => {
                [0 as u8, 0 as u8, 0 as u8, 0 as u8].write_options(writer, endian, args)
            }
        }
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
            }
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
            }
            Vehicle::Unknown => write!(f, "Unknown"),
        }
    }
}
