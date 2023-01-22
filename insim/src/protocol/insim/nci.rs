use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{ConnectionId, RequestId};

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum ILanguage {
    English = 0,
    Deutsch = 1,
    Portuguese = 2,
    French = 3,
    Suomi = 4,
    Norsk = 5,
    Nederlands = 6,
    Catalan = 7,
    Turkish = 8,
    Castellano = 9,
    Italiano = 10,
    Dansk = 11,
    Czech = 12,
    Russian = 13,
    Estonian = 14,
    Serbian = 15,
    Greek = 16,
    Polski = 17,
    Croatian = 18,
    Hungarian = 19,
    Brazilian = 20,
    Swedish = 21,
    Slovak = 22,
    Galego = 23,
    Slovenski = 24,
    Belarussian = 25,
    Latvian = 26,
    Lithuanian = 27,
    TraditionalChinese = 28,
    SimplifiedChinese = 29,
    Japanese = 30,
    Korean = 31,
    Bulgarian = 32,
    Latino = 33,
    Ukrainian = 34,
    Indonesian = 35,
    Romanian = 36,
}

impl Default for ILanguage {
    fn default() -> Self {
        ILanguage::English
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Extra information about the new connection. This is only sent when connected to a game server,
/// and only if an administrative password has been set and used by Insim.
pub struct Nci {
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    #[deku(pad_bytes_after = "3")]
    pub language: ILanguage,

    pub user_id: u32,

    pub ip_addr: u32,
}
