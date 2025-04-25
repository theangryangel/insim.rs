use insim_core::binrw::{self, binrw};

use crate::identifiers::RequestId;

/// Response to a [super::admin_request::Arq] packet, indicating if we are logged in as an administrative user on
/// the selected host.
#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Arp {
    /// Optional request identifier. If a request identifier was sent in the request, it will be
    /// included in any relevant response packet.
    pub reqi: RequestId,

    /// true if we are an admin
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub admin: bool,
}
