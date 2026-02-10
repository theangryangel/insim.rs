use bytes::{Buf, BufMut};
use insim_core::{Decode, DecodeString, Encode, EncodeString, game_version::GameVersion};

use crate::identifiers::RequestId;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Version information about the connected LFS instance.
///
/// - Sent in response to [`TinyType::Ver`](crate::insim::TinyType::Ver).
/// - Also sent on connection if `Isi::reqi` is non-zero.
pub struct Ver {
    /// Request identifier echoed from the request.
    pub reqi: RequestId,

    /// LFS game version string.
    pub version: GameVersion,

    /// Product identifier (e.g., DEMO/S1/S2/S3).
    pub product: String,

    /// InSim protocol version reported by LFS.
    pub insimver: u8,
}

impl Decode for Ver {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf).map_err(|e| e.nested().context("Ver::reqi"))?;
        buf.advance(1);
        let version = GameVersion::decode(buf).map_err(|e| e.nested().context("Ver::version"))?;
        let product =
            String::decode_ascii(buf, 6).map_err(|e| e.nested().context("Ver::product"))?;

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
        self.reqi
            .encode(buf)
            .map_err(|e| e.nested().context("Ver::reqi"))?;
        buf.put_u8(0);
        self.version
            .encode(buf)
            .map_err(|e| e.nested().context("Ver::version"))?;
        self.product
            .encode_codepage(buf, 6, false)
            .map_err(|e| e.nested().context("Ver::product"))?;

        self.insimver
            .encode(buf)
            .map_err(|e| e.nested().context("Ver::insimver"))?;
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
