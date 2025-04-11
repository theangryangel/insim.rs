use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string_until_eof, binrw_write_codepage_string},
    FromToBytes, FromToCodepageBytes,
};

use super::SoundType;
use crate::identifiers::{ConnectionId, PlayerId, RequestId};

const MAX_MTC_TEXT_LEN: usize = 128;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Message to Connection - Send a message to a specific connection, restricted to hosts only
pub struct Mtc {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// See [SoundType].
    pub sound: SoundType,

    /// Unique connection id
    pub ucid: ConnectionId,

    /// Unique player id
    #[brw(pad_after = 2)]
    pub plid: PlayerId,

    /// Message
    #[bw(write_with = binrw_write_codepage_string::<128, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string_until_eof)]
    pub text: String,
}

impl FromToBytes for Mtc {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::from_bytes(buf)?;
        let sound = SoundType::from_bytes(buf)?;
        let ucid = ConnectionId::from_bytes(buf)?;
        let plid = PlayerId::from_bytes(buf)?;
        buf.advance(2);
        let text = String::from_codepage_bytes(buf, MAX_MTC_TEXT_LEN)?;
        Ok(Self {
            reqi,
            sound,
            ucid,
            plid,
            text,
        })
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.to_bytes(buf)?;
        self.sound.to_bytes(buf)?;
        self.ucid.to_bytes(buf)?;
        self.plid.to_bytes(buf)?;
        buf.put_bytes(0, 2);
        self.text
            .to_codepage_bytes_aligned(buf, MAX_MTC_TEXT_LEN, 4)?;
        Ok(())
    }
}

impl_typical_with_request_id!(Mtc);

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use insim_core::binrw::BinWrite;

    use super::*;

    #[test]
    fn test_mtc() {
        let raw = [
            1, // reqi
            1, // soundtype
            0, // ucid
            2, // plid
            0, 0, b'a', b'b', b'c', b'd', b'e', 0, 0, 0,
        ];

        assert_from_to_bytes!(Mtc, raw, |parsed: Mtc| {
            assert_eq!(parsed.reqi, RequestId(1));
            assert_eq!(parsed.plid, PlayerId(2));
            assert_eq!(parsed.ucid, ConnectionId(0));
            assert_eq!(parsed.sound, SoundType::Message);
        });

        let data = Mtc {
            reqi: RequestId(1),
            plid: PlayerId(0),
            ucid: ConnectionId(0),
            sound: SoundType::default(),
            text: "aaaaa".into(),
        };

        let mut buf = Cursor::new(Vec::new());
        let res = data.write_le(&mut buf);
        assert!(res.is_ok());
        let buf = buf.into_inner();

        assert_eq!((buf.len() - 6) % 4, 0);

        assert_eq!(buf.last(), Some(&0));
        assert_eq!(buf.len(), 14);
    }
}
