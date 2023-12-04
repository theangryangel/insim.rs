use insim_core::{identifiers::RequestId, track::Track, binrw::{self, binrw}, string::{binrw_parse_codepage_string, binrw_write_codepage_string}};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

bitflags! {
    /// Bitwise flags used within the [HostInfo] packet, which is in turn used by the [HostList]
    /// packet.
    #[binrw]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct HostInfoFlags: u8 {
         const SPECTATE_PASSWORD_REQUIRED = (1 << 0);
         const LICENSED = (1 << 1);
         const S1 = (1 << 2);
         const S2 = (1 << 3);
         const FIRST = (1 << 6);
         const LAST = (1 << 7);
    }
}

/// Information about a host. Used within the [HostList] packet.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HostInfo {
    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    pub hname: String,

    pub track: Track,

    pub flags: HostInfoFlags,

    pub numconns: u8,
}

/// The relay will send a list of available hosts using this packet. There may be more than one
/// HostList packet sent in response to a [super::host_list_request::HostListRequest]. You may use the [HostInfoFlags] to
/// determine if the host is the last in the list.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HostList {
    pub reqi: RequestId,

    #[bw(calc = hinfo.len() as u8)]
    numhosts: u8,

    #[br(count = numhosts)]
    pub hinfo: Vec<HostInfo>,
}

impl HostList {
    pub fn is_last(&self) -> bool {
        self.hinfo
            .iter()
            .filter(|i| i.flags.contains(HostInfoFlags::LAST))
            .count()
            > 0
    }
}
