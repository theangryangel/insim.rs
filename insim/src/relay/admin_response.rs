use insim_core::{identifiers::RequestId, binrw::{self, binrw}};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Reponse to a [super::admin_request::AdminRequest] packet, indicating if we are logged in as an administrative user on
/// the selected host.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct AdminResponse {
    /// Optional request identifier. If a request identifier was sent in the request, it will be
    /// included in any relevant response packet.
    pub reqi: RequestId,

    pub admin: u8,
}
