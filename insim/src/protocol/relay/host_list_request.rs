use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Request a list of available hosts from the Insim Relay. After sending this packet the relay
/// will respond with a HostList packet.
#[derive(Debug, Clone, Default, InsimEncode, InsimDecode)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HostListRequest {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,
}
