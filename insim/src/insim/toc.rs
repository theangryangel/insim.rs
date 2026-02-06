use crate::identifiers::{ConnectionId, PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Driver swap notification.
///
/// - Indicates a player moved between connections.
pub struct Toc {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player identifier.
    pub plid: PlayerId,

    /// Original connection id.
    pub olducid: ConnectionId,

    /// New connection id for this player.
    #[insim(pad_after = 2)]
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
