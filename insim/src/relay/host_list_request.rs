use insim_core::binrw::{self, binrw};

use crate::identifiers::RequestId;

/// Request a list of available hosts from the Insim Relay. After sending this packet the relay
/// will respond with a [super::host_list::Hos] packet.
#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Hlr {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    #[read_write_buf(pad_after = 1)]
    pub reqi: RequestId,
}
