use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum TinyType {
    #[default]
    None = 0,

    /// Get Version
    Version = 1,

    /// Close
    Close = 2,

    /// External program requesting a reply (Pong)
    Ping = 3,

    /// Reply to a ping
    Pong = 4,

    /// Vote Cancel
    Vtc = 5,

    /// Send camera position
    Scp = 6,

    /// Send state info
    Sst = 7,

    /// Get time in hundredths (i.e. SMALL_RTP)
    Gth = 8,

    /// Multi-player end
    Mpe = 9,

    /// Get multi-player info
    Ism = 10,

    /// Race end
    Ren = 11,

    /// All players cleared from race
    Clr = 12,

    /// Request NCN for all connections
    Ncn = 13,

    /// Request NPL for all players
    Npl = 14,

    /// Get all results
    Res = 15,

    /// Request a Nlp packet
    Nlp = 16,

    /// Request a Mci packet
    Mci = 17,

    /// Request a Reo packet
    Reo = 18,

    /// Request a Rst packet
    Rst = 19,

    /// Request a Axi packet
    Axi = 20,

    /// Autocross cleared
    Axc = 21,

    /// Request a Rip packet
    Rip = 22,

    /// Request a Nci packet for all guests
    Nci = 23,

    /// Request a Small packet, type = Alc
    Alc = 24,

    /// Request a Axm packet, for the entire layout
    Axm = 25,

    /// Request a Slc packet for all connections
    Slc = 26,

    /// Request a Mal packet
    Mal = 27,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// General purpose Tiny packet
pub struct Tiny {
    pub reqi: RequestId,

    pub subt: TinyType,
}
