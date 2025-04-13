use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
    ReadWriteBuf, FromToCodepageBytes,
};

use crate::identifiers::{ConnectionId, RequestId};

bitflags::bitflags! {
    /// Additional facts about this connection. Used within [Ncn].
    #[binrw]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
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

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// New Connection
pub struct Ncn {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection id of new connection
    pub ucid: ConnectionId,

    /// LFS.net username.
    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    pub uname: String,

    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    /// Player Name.
    pub pname: String,

    /// true if administrative user.
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub admin: bool,

    /// Total number of connections now this player has joined, plus host
    pub total: u8,

    #[brw(pad_after = 1)]
    /// Flags describing additional facts about this connection
    pub flags: NcnFlags,
}

impl ReadWriteBuf for Ncn {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        let ucid = ConnectionId::read_buf(buf)?;
        let uname = String::from_codepage_bytes(buf, 24)?;
        let pname = String::from_codepage_bytes(buf, 24)?;
        let admin = u8::read_buf(buf)? > 0;
        let total = u8::read_buf(buf)?;
        let flags = NcnFlags::read_buf(buf)?;
        buf.advance(1);
        Ok(Self {
            reqi,
            ucid,
            uname,
            pname,
            admin,
            total,
            flags,
        })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        self.ucid.write_buf(buf)?;
        self.uname.to_codepage_bytes(buf, 24)?;
        self.pname.to_codepage_bytes(buf, 24)?;
        (self.admin as u8).write_buf(buf)?;
        self.total.write_buf(buf)?;
        self.flags.write_buf(buf)?;
        buf.put_u8(0);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;

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
