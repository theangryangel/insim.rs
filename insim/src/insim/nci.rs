use std::net::Ipv4Addr;

use insim_core::license::License;

use crate::identifiers::{ConnectionId, RequestId};

#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
/// Language
pub enum Language {
    #[default]
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

#[derive(Debug, Clone, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Extra information about the new connection. This is only sent when connected to a game server,
/// and only if an administrative password has been set and used by Insim.
pub struct Nci {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection ID
    pub ucid: ConnectionId,

    /// Language
    pub language: Language,

    /// License level.
    #[read_write_buf(pad_after = 2)]
    pub license: License,

    /// LFS.net player ID
    pub userid: u32,

    /// Originating IP address
    pub ipaddress: Ipv4Addr,
}

impl Default for Nci {
    fn default() -> Self {
        Self {
            reqi: RequestId::default(),
            ucid: ConnectionId::default(),
            language: Language::default(),
            license: License::default(),
            userid: 0,
            ipaddress: Ipv4Addr::new(0, 0, 0, 0),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nci() {
        assert_from_to_bytes!(
            Nci,
            [
                1,   // reqi
                3,   // ucid
                12,  // language
                3,   // license,
                0,   // sp2,
                0,   // sp3,
                3,   // userid (1)
                42,  // userid (2)
                5,   // userid (3)
                1,   // userid (4)
                1,   // ipaddress (1)
                0,   // ipaddress (2)
                0,   // ipaddress (3)
                127, // ipaddress (4)
            ],
            |nci: Nci| {
                assert_eq!(nci.reqi, RequestId(1));
                assert_eq!(nci.ucid, ConnectionId(3));
                assert!(matches!(nci.language, Language::Czech));
                assert_eq!(nci.ipaddress, Ipv4Addr::from([127, 0, 0, 1]));
                assert_eq!(nci.userid, 17115651_u32);
            }
        );
    }
}
