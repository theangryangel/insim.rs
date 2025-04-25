use bitflags::bitflags;
use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
    track::Track,
    ReadWriteBuf,
};

use crate::identifiers::RequestId;

bitflags! {
    /// Provides extended host information
    #[binrw]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    pub struct HostInfoFlags: u8 {
        /// Spectator password is required
         const SPECTATE_PASSWORD_REQUIRED = (1 << 0);
        /// Is the host licensed?
        const LICENSED = (1 << 1);
        /// This host requires a S1 license or greater
        const S1 = (1 << 2);
        /// This host requires a S2 license or greater
        const S2 = (1 << 3);

        /// This is the first of [HostInfo] records
        const FIRST = (1 << 6);
        /// This is the last of [HostInfo] records
        const LAST = (1 << 7);
    }
}

generate_bitflag_helpers!(HostInfoFlags,
    pub requires_spectator_password => SPECTATE_PASSWORD_REQUIRED,
    pub requires_s1 => S1,
    pub requires_s2 => S2,
    pub requires_license => LICENSED,
    pub is_first => FIRST,
    pub is_last => LAST
);

impl_bitflags_from_to_bytes!(HostInfoFlags, u8);

/// Information about a host. Used within the [Hos] packet.
#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct HostInfo {
    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    #[read_write_buf(codepage(length = 32))]
    /// Hostname
    pub hname: String,

    /// Current track
    pub track: Track,

    /// Extended host information, such as license restrictions
    pub flags: HostInfoFlags,

    /// Total number of connections
    pub numconns: u8,
}

/// The relay will send a list of available hosts using this packet. There may be more than one
/// HostList packet sent in response to a [super::host_list_request::Hlr]. You may use the [HostInfoFlags] to
/// determine if the host is the last in the list.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Hos {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    #[bw(calc = hinfo.len() as u8)]
    numhosts: u8,

    /// A partial list of hosts
    #[br(count = numhosts)]
    pub hinfo: Vec<HostInfo>,
}

impl Hos {
    /// Is this the last of all [Hos] packets, for a complete set of hosts?
    pub fn is_last(&self) -> bool {
        self.hinfo.iter().any(|i| i.flags.is_last())
    }
}

impl ReadWriteBuf for Hos {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        let num = u8::read_buf(buf)?;
        let mut hinfo = Vec::with_capacity(num as usize);
        for _i in 0..num {
            hinfo.push(HostInfo::read_buf(buf)?);
        }

        Ok(Self { reqi, hinfo })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        let num = self.hinfo.len() as u8;
        num.write_buf(buf)?;
        for i in self.hinfo.iter() {
            i.write_buf(buf)?;
        }
        Ok(())
    }
}
