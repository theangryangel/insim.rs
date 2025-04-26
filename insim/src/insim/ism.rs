use crate::identifiers::RequestId;

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Insim Multiplayer - LFS sends this when a host is started or joined
pub struct Ism {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[read_write_buf(pad_after = 1)]
    pub reqi: RequestId,

    /// Are we a host? false = guest, true = host
    #[read_write_buf(pad_after = 3)]
    pub host: bool,

    /// Name of server joined/started
    #[read_write_buf(codepage(length = 32))]
    pub hname: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ism() {
        assert_from_to_bytes!(
            Ism,
            [
                1, // reqi
                0, 1, // host
                0, 0, 0, // hname
                b'a', b'B', b'c', b'd', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0
            ],
            |parsed: Ism| {
                assert_eq!(parsed.reqi, RequestId(1));
                assert_eq!(parsed.host, true);
                assert_eq!(&parsed.hname, "aBcd");
            }
        )
    }
}
