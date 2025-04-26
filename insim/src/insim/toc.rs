use crate::identifiers::{ConnectionId, PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Take Over Car - informational - when a 2 connections swap drivers
/// Insim indicates this by sending this packet which describes a transfer of the relationship
/// between this PlayerId and two ConnectionId's.
pub struct Toc {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Players unique Id
    pub plid: PlayerId,

    /// The original connection ID
    pub olducid: ConnectionId,

    /// The new connection ID for this `plid`
    #[read_write_buf(pad_after = 2)]
    pub newucid: ConnectionId,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_toc() {
        assert_from_to_bytes!(
            Toc,
            [
                0, // reqi
                3, // plid
                1, // olducid
                2, // newucid
                0, 0,
            ],
            |toc: Toc| {
                assert_eq!(toc.reqi, RequestId(0));
                assert_eq!(toc.plid, PlayerId(3));
                assert_eq!(toc.olducid, ConnectionId(1));
                assert_eq!(toc.newucid, ConnectionId(2));
            }
        )
    }
}
