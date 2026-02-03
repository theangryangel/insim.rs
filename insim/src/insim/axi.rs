use crate::identifiers::RequestId;

/// AutoX layout summary.
///
/// - Reports counts for objects and checkpoints plus the last loaded layout name.
/// - Can be requested via [`TinyType::Axi`](crate::insim::TinyType::Axi).
#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Axi {
    /// Request identifier echoed by replies.
    #[insim(pad_after = 1)]
    pub reqi: RequestId,

    /// Autocross start position index.
    pub axstart: u8,

    /// Number of checkpoints.
    pub numcp: u8,

    /// Number of objects.
    pub numo: u16,

    /// Name of the last loaded layout (if loaded locally).
    #[insim(codepage(length = 32))]
    pub lname: String,
}

#[cfg(test)]
mod test {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_axi() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&[
            0, // reqi
            0, 2,  // axstart
            3,  // numcp
            23, // numo (1)
            17, // numo (2)
        ]);

        buf.extend_from_slice("test 123".as_bytes());
        buf.put_bytes(0, 32 - 8);

        assert_from_to_bytes!(Axi, buf.freeze(), |axi: Axi| {
            assert_eq!(axi.lname, "test 123");
        });
    }
}
