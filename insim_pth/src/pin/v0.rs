//! LFSPIN file, version 0, revision 0

use bytes::Bytes;
use insim_core::{Decode, Encode};

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
    fn decode(buf: &mut Bytes) -> Result<Self, insim_core::DecodeError> {
        let revision = u8::decode(buf)?;
        if revision > 0 {
            return Err(insim_core::DecodeErrorKind::BadMagic {
                found: Box::new(revision),
            }
            .context("LFSPIN unsupported revision"));
        }

        let _reserved = i32::decode(buf)?;
        let num_configs = u8::decode(buf)?;
        let _reserved1 = u8::decode(buf)?;
        let _reserved2 = u8::decode(buf)?;
        let _reserved3 = u8::decode(buf)?;

        let ms_min_x = i32::decode(buf)?;
        let ms_max_x = i32::decode(buf)?;
        let ms_min_y = i32::decode(buf)?;
        let ms_max_y = i32::decode(buf)?;

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
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        if self.revision > 0 {
            return Err(insim_core::EncodeErrorKind::NoVariantMatch {
                found: self.revision as u64,
            }
            .context("LFSPIN unsupported revision"));
        }

        self.revision.encode(buf)?;
        0i32.encode(buf)?; // reserved
        self.num_configs.encode(buf)?;
        0u8.encode(buf)?; // reserved
        0u8.encode(buf)?; // reserved
        0u8.encode(buf)?; // reserved
        self.ms_min_x.encode(buf)?;
        self.ms_max_x.encode(buf)?;
        self.ms_min_y.encode(buf)?;
        self.ms_max_y.encode(buf)?;

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
        original.encode(&mut buf).expect("Expected to encode");

        let mut bytes = Bytes::from(buf);
        let decoded = LfsPin::decode(&mut bytes).expect("Expected to decode");

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
