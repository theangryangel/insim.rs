use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{PlayerId, RequestId};

use super::CarContact;

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
/// Used within [Hlv] to indicate the hotlap validity failure reason.
pub enum Hlvc {
    Ground = 0,

    Wall = 1,

    Speeding = 4,

    OutOfBounds = 5,
}

impl Default for Hlvc {
    fn default() -> Self {
        Hlvc::Ground
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Reports incidents that would violate Hot Lap Validity checks.
pub struct Hlv {
    pub reqi: RequestId,
    pub plid: PlayerId,
    #[insim(pad_bytes_after = "1")]
    pub hlvc: Hlvc,
    pub time: u16,
    pub c: CarContact,
}
