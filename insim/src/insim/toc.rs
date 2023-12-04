use insim_core::{
    identifiers::{ConnectionId, PlayerId, RequestId},
    binrw::{self, binrw}
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Take Over Car
pub struct Toc {
    pub reqi: RequestId,
    pub plid: PlayerId,

    pub olducid: ConnectionId,
    #[brw(pad_after = 2)]
    pub newucid: ConnectionId,
}
