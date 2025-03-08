use std::str::FromStr;

use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw, BinRead, BinWrite},
    game_version::GameVersion,
    string::{binrw_parse_codepage_string, binrw_write_codepage_string, strip_trailing_nul}, FromToBytes,
};

use crate::identifiers::RequestId;

#[binrw::parser(reader, endian)]
fn parse_game_version() -> binrw::BinResult<GameVersion> {
    let pos = reader.stream_position()?;
    <[u8; 8]>::read_options(reader, endian, ()).and_then(|bytes| {
        std::str::from_utf8(&bytes)
            .map_err(|err| binrw::Error::Custom {
                pos,
                err: Box::new(err),
            })
            .map(|s| {
                GameVersion::from_str(s.trim_end_matches('\0')).map_err(|err| {
                    binrw::Error::Custom {
                        pos,
                        err: Box::new(err),
                    }
                })
            })
    })?
}

#[binrw::writer(writer, endian)]
fn write_game_version(input: &GameVersion) -> binrw::BinResult<()> {
    let mut ver = input.to_string().as_bytes().to_vec();
    if ver.len() > 8 {
        ver.truncate(8);
    } else {
        let remaining = 8 - ver.len();
        if remaining > 0 {
            ver.put_bytes(0, remaining);
        }
    }

    ver.write_options(writer, endian, ())?;

    Ok(())
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Version packet - informational
/// It is advisable to request version information as soon as you have connected, to
/// avoid problems when connecting to a host with a later or earlier version.  You will
/// be sent a version packet on connection if you set ReqI in the IS_ISI packet.
pub struct Ver {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// LFS version, e.g. 0.3G
    #[br(parse_with = parse_game_version)]
    #[bw(write_with = write_game_version)]
    pub version: GameVersion,

    /// Product: DEMO / S1 / S2 / S3
    #[br(parse_with = binrw_parse_codepage_string::<6, _>)]
    #[bw(write_with = binrw_write_codepage_string::<6, _>)]
    pub product: String,

    /// InSim version
    #[brw(pad_after = 1)]
    pub insimver: u8,
}

impl FromToBytes for Ver {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::from_bytes(buf)?;
        buf.advance(1);
        let version = GameVersion::from_bytes(buf)?;
        let product = String::from_utf8_lossy(
            strip_trailing_nul(buf.copy_to_bytes(6).as_ref())
        ).to_string();

        let insimver = buf.get_u8();

        buf.advance(1);

        Ok(Self {
            reqi,
            version,
            product,
            insimver,
        })
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.to_bytes(buf)?;
        buf.put_u8(0);
        self.version.to_bytes(buf)?;

        let product = self.product.as_bytes();
        insim_core::to_bytes_padded!(buf, product, 6);

        self.insimver.to_bytes(buf)?;
        buf.put_u8(0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_version_binrw() {
        let data = vec![
            0, // reqi
            0, // padding
            48, 46, 55, 65, 0, 0, 0, 0, //  game version
            68, 69, 77, 79, 0, 0, // product
            9, // insim ver
            0, // padding
        ];

        let parsed = Ver::read_le(&mut Cursor::new(&data)).unwrap();
        assert_eq!(parsed.version, GameVersion::from_str("0.7A").unwrap());
        assert_eq!(parsed.product, "DEMO");
        assert_eq!(parsed.insimver, 9);

        let mut data2 = Cursor::new(Vec::new());
        parsed.write_le(&mut data2).unwrap();

        assert_eq!(data, data2.into_inner());
    }

    #[test]
    fn test_version() {
        let data = vec![
            0, // reqi
            0, // padding
            48, 46, 55, 65, 0, 0, 0, 0, //  game version
            68, 69, 77, 79, 0, 0, // product
            9, // insim ver
            0, // padding
        ];

        let mut buf = BytesMut::new();
        buf.extend_from_slice(&data);

        let parsed = Ver::from_bytes(&mut buf.freeze()).unwrap();
        assert_eq!(parsed.version, GameVersion::from_str("0.7A").unwrap());
        assert_eq!(parsed.product, "DEMO");
        assert_eq!(parsed.insimver, 9);

        let mut data2 = BytesMut::new();
        parsed.to_bytes(&mut data2).unwrap();

        assert_eq!(&data, data2.as_ref());
    }
}
