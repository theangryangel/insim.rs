use crate::identifiers::RequestId;

/// Auto X Info - Return information about the current layout.
// You can request information about the current layout with this IS_TINY:
// reqi: non-zero (returned in the reply)
// subtype: TINY_AXI (AutoX Info)
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Axi {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[read_write_buf(pad_after = 1)]
    pub reqi: RequestId,

    /// Autocross start position
    pub axstart: u8,

    /// Number of checkpoints
    pub numcp: u8,

    /// Number of objects
    pub numo: u16,

    /// The name of the layout last loaded (if loaded locally)
    #[read_write_buf(codepage(length = 32))]
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
