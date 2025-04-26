use crate::identifiers::RequestId;

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Extended Message (like [Mst](super::Mst), but longer)
pub struct Msx {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[read_write_buf(pad_after = 1)]
    pub reqi: RequestId,

    /// Message
    #[read_write_buf(codepage(length = 96))]
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_msx() {
        let mut data = BytesMut::new();
        data.extend_from_slice(&[
            1, // reqi
            0,
        ]);

        data.extend_from_slice(b"aaaaaa");
        data.put_bytes(0, 96 - 6);

        assert_from_to_bytes!(Msx, data.as_ref(), |msx: Msx| {
            assert_eq!(&msx.msg, "aaaaaa");
        });
    }
}
