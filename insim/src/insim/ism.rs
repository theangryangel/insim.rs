use crate::identifiers::RequestId;

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Multiplayer host information.
///
/// - Sent when a host is started or joined.
/// - Can be requested via [`TinyType::Ism`](crate::insim::TinyType::Ism).
pub struct Ism {
    /// Request identifier echoed by replies.
    #[insim(pad_after = 1)]
    pub reqi: RequestId,

    /// Whether we are the host (true) or a guest (false).
    #[insim(pad_after = 3)]
    pub host: bool,

    /// Host name of the server.
    #[insim(codepage(length = 32))]
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
