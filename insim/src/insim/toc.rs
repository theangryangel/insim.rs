use insim_core::binrw::{self, binrw};

use crate::identifiers::{ConnectionId, PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Take Over Car - informational - when a 2 connections swap drivers
pub struct Toc {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Players unique Id
    pub plid: PlayerId,

    /// The original connection ID
    pub olducid: ConnectionId,

    // The new connection ID
    #[brw(pad_after = 2)]
    pub newucid: ConnectionId,
}
