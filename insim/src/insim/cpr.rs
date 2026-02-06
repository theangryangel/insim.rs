use crate::identifiers::{ConnectionId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Connection rename or plate change notification.
///
/// - Sent when a player changes their name or number plate.
pub struct Cpr {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Connection that changed name or plate.
    pub ucid: ConnectionId,

    #[insim(codepage(length = 24))]
    /// New player name.
    pub pname: String,

    #[insim(codepage(length = 8))]
    /// New number plate.
    pub plate: String,
}

#[cfg(test)]
mod test {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_cpr() {
        let mut raw = BytesMut::new();
        raw.extend_from_slice(&[0, 3]);
        raw.extend_from_slice("user".as_bytes());
        raw.put_bytes(0, 20);
        raw.extend_from_slice("12345678".as_bytes());

        assert_from_to_bytes!(Cpr, raw.as_ref(), |parsed: Cpr| {
            assert_eq!(parsed.pname, "user");
            assert_eq!(parsed.plate, "12345678");
            assert_eq!(parsed.reqi, RequestId(0));
            assert_eq!(parsed.ucid, ConnectionId(3));
        });
    }
}
