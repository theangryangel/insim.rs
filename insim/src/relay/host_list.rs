use bitflags::bitflags;
use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
    track::Track, FromToBytes,
};

use crate::identifiers::RequestId;

bitflags! {
    /// Provides extended host information
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

impl FromToBytes for HostInfoFlags {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let bits = u8::from_bytes(buf)?;
        Ok(Self::from_bits_truncate(bits))
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.bits().to_bytes(buf)
    }
}

/// Information about a host. Used within the [Hos] packet.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct HostInfo {
    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    /// Hostname
    pub hname: String,

    /// Current track
    pub track: Track,

    /// Extended host information, such as license restrictions
    pub flags: HostInfoFlags,

    /// Total number of connections
    pub numconns: u8,
}

impl FromToBytes for HostInfo {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        todo!()
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<usize, insim_core::Error> {
        todo!()
    }
}

/// The relay will send a list of available hosts using this packet. There may be more than one
/// HostList packet sent in response to a [super::host_list_request::Hlr]. You may use the [HostInfoFlags] to
/// determine if the host is the last in the list.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Hos {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// A partial list of hosts
    pub hinfo: Vec<HostInfo>,
}

impl Hos {
    /// Is this the last of all [Hos] packets, for a complete set of hosts?
    pub fn is_last(&self) -> bool {
        self.hinfo.iter().any(|i| i.flags.is_last())
    }
}

impl FromToBytes for Hos {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::from_bytes(buf)?;
        let num = u8::from_bytes(buf)?;
        let mut hinfo = Vec::with_capacity(num as usize);
        for i in 1..=num {
            hinfo.push(HostInfo::from_bytes(buf)?);
        }

        Ok(Self {
            reqi, hinfo
        })
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.to_bytes(buf);
        let num = self.hinfo.len() as u8;
        num.to_bytes(buf)?;
        for i in self.hinfo.iter() {
            i.to_bytes(buf)?;
        }
        Ok(())
    }
}
