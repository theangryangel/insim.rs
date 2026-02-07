use crate::identifiers::{ConnectionId, RequestId};

bitflags::bitflags! {
    /// Additional facts about this connection. Used within [Ncn].
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Connection joined notification.
///
/// - Sent when a connection joins the host.
/// - Can be requested via [`TinyType::Ncn`](crate::insim::TinyType::Ncn).
pub struct Ncn {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Connection identifier for the new connection.
    pub ucid: ConnectionId,

    /// LFS.net username.
    #[insim(codepage(length = 24))]
    pub uname: String,

    #[insim(codepage(length = 24))]
    /// Player nickname.
    pub pname: String,

    /// Whether the connection has admin privileges.
    pub admin: bool,

    /// Total number of connections including host.
    pub total: u8,

    #[insim(pad_after = 1)]
    /// Additional facts about the connection.
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
