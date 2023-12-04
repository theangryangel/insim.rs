use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, PlayerId, RequestId},
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
