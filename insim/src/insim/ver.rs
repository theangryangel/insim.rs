use bytes::{Buf, BufMut};
use insim_core::{game_version::GameVersion, Decode, DecodeString, Encode, EncodeString};

use crate::identifiers::RequestId;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
/// Version packet - informational
/// It is advisable to request version information as soon as you have connected, to
/// avoid problems when connecting to a host with a later or earlier version.  You will
/// be sent a version packet on connection if you set ReqI in the IS_ISI packet.
pub struct Ver {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// LFS version, e.g. 0.3G
    pub version: GameVersion,

    /// Product: DEMO / S1 / S2 / S3
    pub product: String,

    /// InSim version
    pub insimver: u8,
}

impl Decode for Ver {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf)?;
        buf.advance(1);
        let version = GameVersion::decode(buf)?;
        let product = String::decode_ascii(buf, 6)?;

        let insimver = buf.get_u8();

        buf.advance(1);

        Ok(Self {
            reqi,
            version,
            product,
            insimver,
        })
    }
}

impl Encode for Ver {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi.encode(buf)?;
        buf.put_u8(0);
        self.version.encode(buf)?;
        self.product.encode_codepage(buf, 6, false)?;

        self.insimver.encode(buf)?;
        buf.put_u8(0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

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
