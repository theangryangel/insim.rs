use insim_core::binrw::{self, binrw};

use crate::identifiers::RequestId;

/// Ask the relay if we are logged in as an administrative user on the selected host. A
/// [super::admin_response::AdminResponse] is sent back by the relay.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Arq {
    #[brw(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,
}
