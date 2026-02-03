use crate::identifiers::{ConnectionId, PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Message sent by a user with the `/i` command.
///
/// - Delivered to the host's InSim connection.
/// - Contains the raw message text.
pub struct Iii {
    #[insim(pad_after = 1)]
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Connection that sent the message.
    pub ucid: ConnectionId,

    /// Player that sent the message.
    #[insim(pad_after = 2)]
    pub plid: PlayerId,

    /// Message text.
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
        raw.extend_from_slice(b"abcd\0\0\0\0");

        assert_from_to_bytes!(Iii, raw.freeze(), |parsed: Iii| {
            assert_eq!(parsed.reqi, RequestId(0));
            assert_eq!(parsed.ucid, ConnectionId(2));
            assert_eq!(parsed.plid, PlayerId(4));
            assert_eq!(parsed.msg, "abcd");
        });
    }
}
