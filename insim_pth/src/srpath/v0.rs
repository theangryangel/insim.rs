//! SRPTH file, version 0, revision <= 252

use std::ops::{Deref, DerefMut};

use bytes::{Buf, BufMut, Bytes};
use glam::IVec3;
use insim_core::{Decode, Encode};

use crate::node::Node;

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    /// SrPath0Flags
    pub struct SrPathFlags: u32 {
        /// Unknown
        const LOOP = (1 << 0);
        /// Rect
        const RECT = (1 << 1);
        /// Route
        const ROUTE = (1 << 2);
        /// Allow flip?
        const ALLOW_FLIP = (1 << 3);
    }
}

impl Encode for SrPathFlags {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        let val = self.bits();
        val.encode(buf)?;
        Ok(())
    }
}

impl Decode for SrPathFlags {
    fn decode(buf: &mut Bytes) -> Result<Self, insim_core::DecodeError> {
        let val = u32::decode(buf)?;
        Ok(Self::from_bits_truncate(val))
    }
}

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    /// SrNodeFlags
    pub struct SrNodeFlags: u8 {
        /// ??
        const LEFT = (1 << 0);
        /// ??
        const RIGHT = (1 << 1);
        /// ??
        const HIGH = (1 << 2);
        /// ??
        const WALLED = (1 << 3);
    }
}

impl Encode for SrNodeFlags {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        let val = self.bits();
        val.encode(buf)?;
        Ok(())
    }
}

impl Decode for SrNodeFlags {
    fn decode(buf: &mut Bytes) -> Result<Self, insim_core::DecodeError> {
        let val = u8::decode(buf)?;
        Ok(Self::from_bits_truncate(val))
    }
}

#[derive(Debug, Default, Clone, PartialEq, insim_core::Decode, insim_core::Encode)]
/// SRPATH node
pub struct SrNode {
    /// Node flags
    #[insim(pad_after = 3)]
    pub flags: SrNodeFlags,
    /// Node information, xyz, limits, etc.
    pub node: Node,
}

// Allow automatic access to anything in node
impl Deref for SrNode {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl DerefMut for SrNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

#[derive(Debug, Default, Clone, PartialEq, insim_core::Decode, insim_core::Encode)]
/// Where is pole position located?
pub struct SrPolePosition {
    /// x,y,z
    pub xyz: IVec3,
    /// heading
    pub heading: f32,
}

#[derive(Debug, Default, PartialEq)]
/// SRPTH file, version 0, revision <= 252
pub struct SrPth {
    /// Original revsion
    pub revision: u8,

    /// Path flags
    pub flags: SrPathFlags,

    /// mini_rev
    pub mini_rev: u8,

    /// Which node is the finishing line
    pub split0_node: usize,

    /// Which node is split 1
    pub split1_node: usize,

    /// Which node is split 2
    pub split2_node: usize,

    /// Which node is split 3
    pub split3_node: usize,

    /// Pole position
    pub pole_position: SrPolePosition,

    /// A list of main track nodes
    pub main_nodes: Vec<SrNode>,

    /// A list of pit blocks
    pub pit0_nodes: Vec<SrNode>,

    /// A list of alt pit blocks
    pub pit1_nodes: Vec<SrNode>,
}

impl Decode for SrPth {
    fn decode(buf: &mut Bytes) -> Result<Self, insim_core::DecodeError> {
        let revision = u8::decode(buf)?;
        if revision > 252 {
            return Err(insim_core::DecodeError::BadMagic {
                found: Box::new(revision),
            });
        }

        let flags = SrPathFlags::decode(buf)?;
        let mini_rev = u8::decode(buf)?;

        if mini_rev > 9 {
            return Err(insim_core::DecodeError::BadMagic {
                found: Box::new(mini_rev),
            });
        }

        buf.advance(3);

        let num_main_nodes = u16::decode(buf)?;
        let num_pit0_nodes = u16::decode(buf)?;
        let num_pit1_nodes = u16::decode(buf)?;

        buf.advance(2);

        let split0_node = u32::decode(buf)? as usize;
        let split1_node = u32::decode(buf)? as usize;
        let split2_node = u32::decode(buf)? as usize;
        let split3_node = u32::decode(buf)? as usize;

        let pole_position = SrPolePosition::decode(buf)?;

        let main_nodes: Vec<_> = (0..num_main_nodes)
            .map(|_| SrNode::decode(buf))
            .collect::<Result<Vec<_>, _>>()?;

        let pit0_nodes: Vec<_> = (0..num_pit0_nodes)
            .map(|_| SrNode::decode(buf))
            .collect::<Result<Vec<_>, _>>()?;

        let pit1_nodes: Vec<_> = (0..num_pit1_nodes)
            .map(|_| SrNode::decode(buf))
            .collect::<Result<Vec<_>, _>>()?;

        assert_eq!(main_nodes.len(), num_main_nodes as usize);
        assert_eq!(pit0_nodes.len(), num_pit0_nodes as usize);
        assert_eq!(pit1_nodes.len(), num_pit1_nodes as usize);

        Ok(Self {
            revision,
            flags,
            mini_rev,
            split0_node,
            split1_node,
            split2_node,
            split3_node,
            pole_position,
            main_nodes,
            pit0_nodes,
            pit1_nodes,
        })
    }
}

impl Encode for SrPth {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        if self.revision > 252 {
            return Err(insim_core::EncodeError::NoVariantMatch {
                found: self.revision as u64,
            });
        }
        if self.main_nodes.len() > (u16::MAX as usize) {
            return Err(insim_core::EncodeError::TooLarge);
        }
        if self.pit0_nodes.len() > (u16::MAX as usize) {
            return Err(insim_core::EncodeError::TooLarge);
        }
        if self.pit1_nodes.len() > (u16::MAX as usize) {
            return Err(insim_core::EncodeError::TooLarge);
        }

        self.revision.encode(buf)?;
        self.flags.encode(buf)?;

        self.mini_rev.encode(buf)?;
        buf.put_bytes(0, 3);

        (self.main_nodes.len() as u16).encode(buf)?;
        (self.pit0_nodes.len() as u16).encode(buf)?;
        (self.pit1_nodes.len() as u16).encode(buf)?;
        buf.put_bytes(0, 2);

        (self.split0_node as u32).encode(buf)?;
        (self.split1_node as u32).encode(buf)?;
        (self.split2_node as u32).encode(buf)?;
        (self.split3_node as u32).encode(buf)?;

        self.pole_position.encode(buf)?;

        for i in self
            .main_nodes
            .iter()
            .chain(self.pit0_nodes.iter())
            .chain(self.pit1_nodes.iter())
        {
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

    fn assert_valid_as1_pth(p: &SrPth) {
        assert_eq!(p.revision, 252);
    }

    #[test]
    fn test_decode_from_pathbuf() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS1_SRPATHv0r252.pth");
        let p = Pth::from_path(&path).expect("Expected PTH file to be parsed");

        match p {
            Pth::SrPath0(as1) => assert_valid_as1_pth(&as1),
            _ => panic!("Expected SRPATH file"),
        }
    }

    #[test]
    fn test_decode_from_file() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS1_SRPATHv0r252.pth");
        let mut file = fs::File::open(path).expect("Expected Autocross_3DH.smx to exist");
        let p = Pth::read(&mut file).expect("Expected PTH file to be parsed");

        match p {
            Pth::SrPath0(as1) => {
                assert_valid_as1_pth(&as1);
            },
            _ => panic!("Expected SRPATH file"),
        }
    }

    #[test]
    fn test_encode_identical() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./tests/AS1_SRPATHv0r252.pth");
        let p = Pth::from_path(&path).expect("Expected PTH file to be parsed");

        let mut file = fs::File::open(path).expect("Expected AS1.pth to exist");
        let mut raw: Vec<u8> = Vec::new();
        let _ = file
            .read_to_end(&mut raw)
            .expect("Expected to read whole file");

        match &p {
            Pth::SrPath0(as1) => {
                assert_valid_as1_pth(&as1);
                let mut inner = Vec::new();
                let written = p.write(&mut inner).expect("Expected to write");

                assert_eq!(&inner[0..=8], &raw[0..=8]);
                assert_eq!(written, raw.len());
                assert_eq!(&inner, &raw);
            },
            _ => panic!("Expected SRPATH file"),
        }
    }
}
