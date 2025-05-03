use crate::identifiers::RequestId;

/// Ask the relay if we are logged in as an administrative user on the selected host. A
/// [super::admin_response::Arp] is sent back by the relay.
#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Arq {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[insim(pad_after = 1)]
    pub reqi: RequestId,
}
