use insim_core::{identifiers::RequestId, prelude::*, ser::Limit, track::Track};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

bitflags! {
    /// Bitwise flags used within the [HostInfo] packet, which is in turn used by the [HostList]
    /// packet.
    #[derive(Default)]
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

impl Decodable for HostInfoFlags {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Default,
    {
        Ok(Self::from_bits_truncate(u8::decode(buf, limit)?))
    }
}

impl Encodable for HostInfoFlags {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<(), insim_core::EncodableError>
    where
        Self: Sized,
    {
        self.bits().encode(buf, limit)?;
        Ok(())
    }
}

/// Information about a host. Used within the [HostList] packet.
#[derive(Debug, Clone, Default, InsimEncode, InsimDecode)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HostInfo {
    #[insim(bytes = "32")]
    pub hname: String,

    pub track: Track,

    pub flags: HostInfoFlags,

    pub numconns: u8,
}

/// The relay will send a list of available hosts using this packet. There may be more than one
/// HostList packet sent in response to a [super::host_list_request::HostListRequest]. You may use the [HostInfoFlags] to
/// determine if the host is the last in the list.
#[derive(Debug, Clone, Default, InsimEncode, InsimDecode)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HostList {
    pub reqi: RequestId,

    pub numhosts: u8,

    #[insim(count = "numhosts")]
    pub hinfo: Vec<HostInfo>,
}
