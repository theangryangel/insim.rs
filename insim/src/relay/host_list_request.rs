use crate::identifiers::RequestId;

/// Request a list of available hosts from the Insim Relay. After sending this packet the relay
/// will respond with a [super::host_list::Hos] packet.
#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Hlr {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[read_write_buf(pad_after = 1)]
    pub reqi: RequestId,
}
