use crate::identifiers::RequestId;

/// Enum for the sound field of [Msl].
#[derive(
    Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, insim_core::Decode, insim_core::Encode,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Send a message to the local computer only. If you are connected to a server this means the
/// console. If you are connected to a client this means to the local client only.
pub struct Msl {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// See [SoundType]
    pub sound: SoundType,

    /// Message
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
