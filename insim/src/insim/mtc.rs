use super::SoundType;
use crate::identifiers::{ConnectionId, PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Send a message to a specific connection (host only).
///
/// - Can target a connection or a player.
/// - Supports a sound effect on delivery.
pub struct Mtc {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Sound effect to play with the message.
    pub sound: SoundType,

    /// Connection to receive the message (0 = host, 255 = all).
    pub ucid: ConnectionId,

    /// Player to receive the message (0 = use `ucid`).
    #[insim(pad_after = 2)]
    pub plid: PlayerId,

    /// Message text.
    #[insim(codepage(length = 128, align_to = 4, trailing_nul = true))]
    pub text: String,
}

impl_typical_with_request_id!(Mtc);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mtc() {
        let raw = [
            1, // reqi
            1, // soundtype
            0, // ucid
            2, // plid
            0, 0, b'a', b'b', b'c', b'd', b'e', 0, 0, 0,
        ];

        assert_from_to_bytes!(Mtc, raw, |parsed: Mtc| {
            assert_eq!(parsed.reqi, RequestId(1));
            assert_eq!(parsed.plid, PlayerId(2));
            assert_eq!(parsed.ucid, ConnectionId(0));
            assert_eq!(parsed.sound, SoundType::Message);
            assert_eq!(&parsed.text, "abcde");
        });
    }
}
