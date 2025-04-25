use std::str::FromStr;

use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw, BinRead, BinWrite},
    game_version::GameVersion,
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
    FromToAsciiBytes, FromToCodepageBytes, ReadWriteBuf,
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

impl ReadWriteBuf for Ver {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        buf.advance(1);
        let version = GameVersion::read_buf(buf)?;
        let product = String::from_ascii_bytes(buf, 6)?;

        let insimver = buf.get_u8();

        buf.advance(1);

        Ok(Self {
            reqi,
            version,
            product,
            insimver,
        })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        buf.put_u8(0);
        self.version.write_buf(buf)?;
        self.product.to_codepage_bytes(buf, 6)?;

        self.insimver.write_buf(buf)?;
        buf.put_u8(0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_from_to_bytes!(
            Ver,
            vec![
                0, // reqi
                0, // padding
                48, 46, 55, 65, 0, 0, 0, 0, //  game version
                68, 69, 77, 79, 0, 0, // product
                9, // insim ver
                0, // padding
            ],
            |parsed: Ver| {
                assert_eq!(parsed.version, GameVersion::from_str("0.7A").unwrap());
                assert_eq!(parsed.product, "DEMO");
                assert_eq!(parsed.insimver, 9);
            }
        );
    }
}
