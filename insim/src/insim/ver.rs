use insim_core::{Decode, DecodeContext, Encode, EncodeContext, game_version::GameVersion};

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
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let reqi = ctx.decode::<RequestId>("reqi")?;
        ctx.pad("zero", 1)?;
        let version = ctx.decode::<GameVersion>("version")?;
        let product = ctx.decode_ascii("product", 6)?;

        let insimver = ctx.decode::<u8>("insimver")?;

        ctx.pad("sp0", 1)?;

        Ok(Self {
            reqi,
            version,
            product,
            insimver,
        })
    }
}

impl Encode for Ver {
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.encode("reqi", &self.reqi)?;
        ctx.pad("zero", 1)?;
        ctx.encode("version", &self.version)?;
        ctx.encode_ascii("product", &self.product, 6, false)?;

        ctx.encode("insimver", &self.insimver)?;
        ctx.pad("sp0", 1)?;

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
