use crate::identifiers::{ConnectionId, PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// InsIm Info -  a /i message from user to hosts Insim
pub struct Iii {
    #[insim(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection ID that the message was received from
    pub ucid: ConnectionId,

    /// Unique player iD that the message was received from
    #[insim(pad_after = 2)]
    pub plid: PlayerId,

    /// The message
    #[insim(codepage(length = 64, align_to = 4, trailing_nul = true))]
    pub msg: String,
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_iii() {
        let mut raw = BytesMut::new();
        raw.extend_from_slice(&[
            0, // reqi
            0, // zero
            2, // ucid
            4, // plid
            0, // sp2
            0, // sp3
        ]);
        raw.extend_from_slice(b"abcd");

        assert_from_to_bytes!(Iii, raw.freeze(), |parsed: Iii| {
            assert_eq!(parsed.reqi, RequestId(0));
            assert_eq!(parsed.ucid, ConnectionId(2));
            assert_eq!(parsed.plid, PlayerId(4));
            assert_eq!(parsed.msg, "abcd");
        });
    }
}
