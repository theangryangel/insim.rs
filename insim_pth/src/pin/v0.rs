//! LFSPIN file, version 0, revision 0

use insim_core::{Decode, DecodeContext, Encode, EncodeContext};

#[derive(Debug, Default, PartialEq)]
/// PIN file - Path info file for Live for Speed 0.8A
pub struct LfsPin {
    /// Original revision
    pub revision: u8,

    /// Number of pin configurations
    pub num_configs: u8,

    /// Minimum X coordinate on minimap
    pub ms_min_x: i32,

    /// Maximum X coordinate on minimap
    pub ms_max_x: i32,

    /// Minimum Y coordinate on minimap
    pub ms_min_y: i32,

    /// Maximum Y coordinate on minimap
    pub ms_max_y: i32,
}

impl Decode for LfsPin {
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let revision = ctx.decode::<u8>("revision")?;
        if revision > 0 {
            return Err(insim_core::DecodeErrorKind::BadMagic {
                found: Box::new(revision),
            }
            .context("LFSPIN unsupported revision"));
        }

        ctx.pad("reserved", 4)?;
        let num_configs = ctx.decode::<u8>("num_configs")?;
        ctx.pad("reserved1", 1)?;
        ctx.pad("reserved2", 1)?;
        ctx.pad("reserved3", 1)?;

        let ms_min_x = ctx.decode::<i32>("ms_min_x")?;
        let ms_max_x = ctx.decode::<i32>("ms_max_x")?;
        let ms_min_y = ctx.decode::<i32>("ms_min_y")?;
        let ms_max_y = ctx.decode::<i32>("ms_max_y")?;

        Ok(Self {
            revision,
            num_configs,
            ms_min_x,
            ms_max_x,
            ms_min_y,
            ms_max_y,
        })
    }
}

impl Encode for LfsPin {
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        if self.revision > 0 {
            return Err(insim_core::EncodeErrorKind::NoVariantMatch {
                found: self.revision as u64,
            }
            .context("LFSPIN unsupported revision"));
        }

        ctx.encode("revision", &self.revision)?;
        ctx.pad("reserved", 4)?;
        ctx.encode("num_configs", &self.num_configs)?;
        ctx.pad("reserved1", 1)?;
        ctx.pad("reserved2", 1)?;
        ctx.pad("reserved3", 1)?;
        ctx.encode("ms_min_x", &self.ms_min_x)?;
        ctx.encode("ms_max_x", &self.ms_max_x)?;
        ctx.encode("ms_min_y", &self.ms_min_y)?;
        ctx.encode("ms_max_y", &self.ms_max_y)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{fs, io::Read, path::PathBuf};

    use super::*;
    use crate::Pin;

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = LfsPin {
            revision: 0,
            num_configs: 1,
            ms_min_x: -100,
            ms_max_x: 100,
            ms_min_y: -50,
            ms_max_y: 50,
        };

        let mut buf = bytes::BytesMut::new();
        original
            .encode(&mut EncodeContext::new(&mut buf))
            .expect("Expected to encode");

        let mut bytes = bytes::Bytes::from(buf);
        let decoded =
            LfsPin::decode(&mut DecodeContext::new(&mut bytes)).expect("Expected to decode");

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_decode_from_pathbuf() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS_LFSPINv0r0.pin");
        let p = Pin::from_path(&path).expect("Expected PIN file to be parsed");

        let Pin::LfsPin0(as_pin) = p;
        assert_eq!(as_pin.revision, 0);
    }

    #[test]
    fn test_decode_from_file() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS_LFSPINv0r0.pin");
        let mut file = fs::File::open(path).expect("Expected AS_LFSPINv0r0.pin to exist");
        let p = Pin::read(&mut file).expect("Expected PIN file to be parsed");

        let Pin::LfsPin0(as_pin) = p;
        assert_eq!(as_pin.revision, 0);
    }

    #[test]
    fn test_encode_identical() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS_LFSPINv0r0.pin");
        let p = Pin::from_path(&path).expect("Expected PIN file to be parsed");

        let mut file = fs::File::open(path).expect("Expected AS_LFSPINv0r0.pin to exist");
        let mut raw: Vec<u8> = Vec::new();
        let _ = file
            .read_to_end(&mut raw)
            .expect("Expected to read whole file");

        let Pin::LfsPin0(as_pin) = &p;
        assert_eq!(as_pin.revision, 0);
        let mut inner = Vec::new();
        let written = p.write(&mut inner).expect("Expected to write");

        assert_eq!(written, raw.len());
        assert_eq!(&inner[0..=8], &raw[0..=8]);
        assert_eq!(&inner, &raw);
    }
}
