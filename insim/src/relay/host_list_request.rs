use insim_core::{
    binrw::{self, binrw},
    identifiers::RequestId,
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Request a list of available hosts from the Insim Relay. After sending this packet the relay
/// will respond with a HostList packet.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HostListRequest {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,
}
