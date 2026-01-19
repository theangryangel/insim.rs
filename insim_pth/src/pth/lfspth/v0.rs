//! LFSPATH file, version 0, revision <= 0

use bytes::Bytes;
use insim_core::{Decode, Encode};

use crate::node::Node;

#[derive(Debug, Default, PartialEq)]
/// PTH file
pub struct LfsPth {
    /// Original revsion
    pub revision: u8,

    /// Which node is the finishing line
    pub finish_line_node: i32,

    /// A list of nodes
    pub nodes: Vec<Node>,
}

impl Decode for LfsPth {
    fn decode(buf: &mut Bytes) -> Result<Self, insim_core::DecodeError> {
        let revision = u8::decode(buf)?;
        if revision > 0 {
            return Err(insim_core::DecodeErrorKind::OutOfRange {
                min: 0,
                max: 0,
                found: revision as usize,
            }
            .context("LFSPTH unsupported revision"));
        }
        let num_nodes = i32::decode(buf)?;
        let finish_line_node = i32::decode(buf)?;
        let nodes: Vec<_> = (0..num_nodes)
            .map(|_| Node::decode(buf))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            revision,
            finish_line_node,
            nodes,
        })
    }
}

impl Encode for LfsPth {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        if self.revision > 0 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 0,
                found: self.revision as usize,
            }
            .context("LFSPTH unsupported revision"));
        }
        self.revision.encode(buf)?;
        if self.nodes.len() > (i32::MAX as usize) {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: i32::MAX as usize,
                found: self.nodes.len(),
            }
            .context("LFSPTH too many nodes"));
        }
        (self.nodes.len() as i32).encode(buf)?;
        self.finish_line_node.encode(buf)?;
        for i in self.nodes.iter() {
            i.encode(buf)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{fs, io::Read, path::PathBuf};

    use super::*;
    use crate::Pth;

    fn assert_valid_as1_pth(p: &LfsPth) {
        assert_eq!(p.finish_line_node, 250);
    }

    #[test]
    fn test_lfspthv0r0_decode_from_pathbuf() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS1_LFSPTHv0r0.pth");
        let p = Pth::from_path(&path).expect("Expected PTH file to be parsed");

        match p {
            Pth::LfsPth0(as1) => {
                assert_eq!(as1.revision, 0);
                assert_valid_as1_pth(&as1)
            },
            _ => panic!("Expected LFSPTH file"),
        }
    }

    #[test]
    fn test_lfspthpthv0r0_decode_from_file() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS1_LFSPTHv0r0.pth");
        let mut file = fs::File::open(path).expect("Expected Autocross_3DH.smx to exist");
        let p = Pth::read(&mut file).expect("Expected PTH file to be parsed");

        match p {
            Pth::LfsPth0(as1) => {
                assert_eq!(as1.revision, 0);
                assert_valid_as1_pth(&as1);
            },
            _ => panic!("Expected LFSPTH file"),
        }
    }

    #[test]
    fn test_lfspthv0r0_encode_identical() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS1_LFSPTHv0r0.pth");
        let p = Pth::from_path(&path).expect("Expected PTH file to be parsed");

        let mut file = fs::File::open(path).expect("Expected AS1.pth to exist");
        let mut raw: Vec<u8> = Vec::new();
        let _ = file
            .read_to_end(&mut raw)
            .expect("Expected to read whole file");

        match &p {
            Pth::LfsPth0(as1) => {
                assert_eq!(as1.revision, 0);
                assert_valid_as1_pth(&as1);
                let mut inner = Vec::new();
                let written = p.write(&mut inner).expect("Expected to write");

                assert_eq!(written, raw.len());
                assert_eq!(&inner[0..=8], &raw[0..=8]);
                assert_eq!(&inner, &raw);
            },
            _ => panic!("Expected LFSPTH file"),
        }
    }
}
