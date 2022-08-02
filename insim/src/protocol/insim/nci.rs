use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::ConnectionId;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    type = "u8",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub enum ILanguage {
    #[deku(id = "0")]
    English,
    #[deku(id = "1")]
    Deutsch,
    #[deku(id = "2")]
    Portuguese,
    #[deku(id = "3")]
    French,
    #[deku(id = "4")]
    Suomi,
    #[deku(id = "5")]
    Norsk,
    #[deku(id = "6")]
    Nederlands,
    #[deku(id = "7")]
    Catalan,
    #[deku(id = "8")]
    Turkish,
    #[deku(id = "9")]
    Castellano,
    #[deku(id = "10")]
    Italiano,
    #[deku(id = "11")]
    Dansk,
    #[deku(id = "12")]
    Czech,
    #[deku(id = "13")]
    Russian,
    #[deku(id = "14")]
    Estonian,
    #[deku(id = "15")]
    Serbian,
    #[deku(id = "16")]
    Greek,
    #[deku(id = "17")]
    Polski,
    #[deku(id = "18")]
    Croatian,
    #[deku(id = "19")]
    Hungarian,
    #[deku(id = "20")]
    Brazilian,
    #[deku(id = "21")]
    Swedish,
    #[deku(id = "22")]
    Slovak,
    #[deku(id = "23")]
    Galego,
    #[deku(id = "24")]
    Slovenski,
    #[deku(id = "25")]
    Belarussian,
    #[deku(id = "26")]
    Latvian,
    #[deku(id = "27")]
    Lithuanian,
    #[deku(id = "28")]
    TraditionalChinese,
    #[deku(id = "29")]
    SimplifiedChinese,
    #[deku(id = "30")]
    Japanese,
    #[deku(id = "31")]
    Korean,
    #[deku(id = "32")]
    Bulgarian,
    #[deku(id = "33")]
    Latino,
    #[deku(id = "34")]
    Ukrainian,
    #[deku(id = "35")]
    Indonesian,
    #[deku(id = "36")]
    Romanian,
}

impl Default for ILanguage {
    fn default() -> Self {
        ILanguage::English
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Extra information about the new connection. This is only sent when connected to a game server,
/// and only if an administrative password has been set and used by Insim.
pub struct Nci {
    pub reqi: u8,

    pub ucid: ConnectionId,

    #[deku(pad_bytes_after = "3")]
    pub language: ILanguage,

    pub user_id: u32,

    pub ip_addr: u32,
}
