use crate::identifiers::{ConnectionId, RequestId};

bitflags::bitflags! {
    /// Additional facts about this connection. Used within [Ncn].
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    pub struct NcnFlags: u8 {
        /// User is remote
        const REMOTE = (1 << 2);
    }
}

generate_bitflag_helpers! {
    NcnFlags,
    pub is_remote => REMOTE
}

impl_bitflags_from_to_bytes!(NcnFlags, u8);

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// New Connection
pub struct Ncn {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection id of new connection
    pub ucid: ConnectionId,

    /// LFS.net username.
    #[read_write_buf(codepage(length = 24))]
    pub uname: String,

    #[read_write_buf(codepage(length = 24))]
    /// Player Name.
    pub pname: String,

    /// true if administrative user.
    pub admin: bool,

    /// Total number of connections now this player has joined, plus host
    pub total: u8,

    #[read_write_buf(pad_after = 1)]
    /// Flags describing additional facts about this connection
    pub flags: NcnFlags,
}

#[cfg(test)]
mod test {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_ncn() {
        let mut raw = BytesMut::new();
        raw.extend_from_slice(&[
            2, // reqi
            3, // ucid
        ]);

        raw.extend_from_slice("user".as_bytes());
        raw.put_bytes(0, 20);
        raw.extend_from_slice("player".as_bytes());
        raw.put_bytes(0, 18);

        raw.extend_from_slice(&[
            1,  // admin
            14, // total
            4,  // flags
            0,  // sp3
        ]);

        assert_from_to_bytes!(Ncn, raw.as_ref(), |parsed: Ncn| {
            assert!(parsed.admin);
            assert_eq!(parsed.total, 14);
            assert_eq!(parsed.reqi, RequestId(2));
            assert_eq!(parsed.ucid, ConnectionId(3));
            assert!(parsed.flags.is_remote());
            assert_eq!(parsed.uname, "user");
            assert_eq!(parsed.pname, "player");
        });
    }
}
