use insim_core::binrw::{self, binrw};

use crate::identifiers::RequestId;

/// Ask the relay if we are logged in as an administrative user on the selected host. A
/// [super::admin_response::Arp] is sent back by the relay.
#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Arq {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[read_write_buf(pad_after = 1)]
    #[brw(pad_after = 1)]
    pub reqi: RequestId,
}
