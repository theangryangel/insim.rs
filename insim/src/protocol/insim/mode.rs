use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Screen Mode (referred to as originally IS_MOD within Insim.txt)
pub struct Mode {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    /// Set to choose 16-bit
    pub bit16: i8,

    /// Refresh rate, zero for default
    pub rr: i8,

    /// Screen width. Zero to switch to windowed mode.
    pub width: i8,

    /// Screen height. Zero to switch to windowed mode.
    pub height: i8,
}
