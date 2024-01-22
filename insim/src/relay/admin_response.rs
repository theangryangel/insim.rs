use insim_core::binrw::{self, binrw};

use crate::identifiers::RequestId;

/// Reponse to a [super::admin_request::AdminRequest] packet, indicating if we are logged in as an administrative user on
/// the selected host.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AdminResponse {
    /// Optional request identifier. If a request identifier was sent in the request, it will be
    /// included in any relevant response packet.
    pub reqi: RequestId,

    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    /// true if we are an admin
    pub admin: bool,
}
