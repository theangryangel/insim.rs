use crate::identifiers::RequestId;

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Send a message to LFS as if typed by a user.
///
/// - Supports chat messages and slash commands.
pub struct Mst {
    /// Request identifier echoed by replies.
    #[insim(pad_after = 1)]
    pub reqi: RequestId,

    /// Message text.
    #[insim(codepage(length = 64, trailing_nul = true))]
    pub msg: String,
}

impl_typical_with_request_id!(Mst);

impl Mst {
    /// Send `msg` as if typed by the host: chat, or a command such as `/end`.
    /// LFS takes at most 63 characters here; use [`Msx`](super::Msx) for
    /// longer messages.
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_mst() {
        let mut raw = BytesMut::new();
        raw.extend_from_slice(&[1, 0, b'a', b'b', b'c', b'd', b'e', b'f']);
        raw.put_bytes(0, 64 + 2 - raw.len());

        assert_from_to_bytes!(Mst, raw.as_ref(), |parsed: Mst| {
            assert_eq!(parsed.msg, "abcdef".to_string());
        });
    }
}
