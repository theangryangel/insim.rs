use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Screen Mode (referred to as originally IS_MOD within Insim.txt)
pub struct Mode {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    /// Set to choose 16-bit
    pub bit16: i8,

    /// Refresh rate, zero for default
    pub rr: i8,

    /// Screen width. Zero to switch to windowed mode.
    pub width: i8,

    /// Screen height. Zero to switch to windowed mode.
    pub height: i8,
}
