use crate::identifiers::RequestId;

/// Sound effect used when delivering [Msl] or [Mtc](super::Mtc) messages.
#[derive(
    Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, insim_core::Decode, insim_core::Encode,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[non_exhaustive]
pub enum SoundType {
    #[default]
    /// Silent
    Silent = 0,

    /// Message "ping"
    Message = 1,

    /// System message "ping"
    SysMessage = 2,

    /// Invalid key "ping"
    InvalidKey = 3,

    /// Error "ping"
    Error = 4,
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Send a message to the local computer only.
///
/// - On a dedicated host this appears in the host console.
/// - On a client this appears only to the local player.
pub struct Msl {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Sound effect to play with the message.
    pub sound: SoundType,

    /// Message text.
    #[insim(codepage(length = 128, trailing_nul = true))]
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_msl() {
        let mut data = BytesMut::new();
        data.extend_from_slice(&[
            1, // reqi
            0, // sound
        ]);

        data.extend_from_slice(b"aaaaaa");
        data.put_bytes(0, 122);

        assert_from_to_bytes!(Msl, data.freeze(), |msl: Msl| {
            assert_eq!(&msl.msg, "aaaaaa");
        });
    }
}
