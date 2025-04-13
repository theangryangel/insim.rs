use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
    FromToCodepageBytes, ReadWriteBuf,
};

use crate::identifiers::{ConnectionId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Connection Player Renamed indicates that a player has changed their name or number plate.
pub struct Cpr {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection ID of the connection that was renamed
    pub ucid: ConnectionId,

    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    /// New player name
    pub pname: String,

    #[br(parse_with = binrw_parse_codepage_string::<8, _>)]
    #[bw(write_with = binrw_write_codepage_string::<8, _>)]
    /// New number plate
    pub plate: String,
}

impl ReadWriteBuf for Cpr {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        let ucid = ConnectionId::read_buf(buf)?;
        let pname = String::from_codepage_bytes(buf, 24)?;
        let plate = String::from_codepage_bytes(buf, 8)?;
        Ok(Self {
            reqi,
            ucid,
            pname,
            plate,
        })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        self.ucid.write_buf(buf)?;
        self.pname.to_codepage_bytes(buf, 24)?;
        self.plate.to_codepage_bytes(buf, 8)?;
        Ok(())
    }
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
