//! SRPTH file, version 0, revision <= 252

use std::ops::{Deref, DerefMut};

use insim_core::{Decode, DecodeContext, Encode, EncodeContext};

use crate::node::{Node, NodeCoordinate};

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
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.encode("bits", &self.bits())
    }
}

impl Decode for SrPathFlags {
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        ctx.decode::<u32>("bits").map(Self::from_bits_truncate)
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
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.encode("bits", &self.bits())
    }
}

impl Decode for SrNodeFlags {
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        ctx.decode::<u8>("bits").map(Self::from_bits_truncate)
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
    pub xyz: NodeCoordinate,
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
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let revision = ctx.decode::<u8>("revision")?;
        if revision > 252 {
            return Err(insim_core::DecodeErrorKind::OutOfRange {
                min: 0,
                max: 252,
                found: revision as usize,
            }
            .context("SRPATH unsupported revision"));
        }

        let flags = ctx.decode::<SrPathFlags>("flags")?;
        let mini_rev = ctx.decode::<u8>("mini_rev")?;

        if mini_rev > 9 {
            return Err(insim_core::DecodeErrorKind::OutOfRange {
                min: 0,
                max: 9,
                found: mini_rev as usize,
            }
            .context("SRPATH unsupported mini_rev"));
        }

        ctx.pad("reserved", 3)?;

        let num_main_nodes = ctx.decode::<u16>("num_main_nodes")?;
        let num_pit0_nodes = ctx.decode::<u16>("num_pit0_nodes")?;
        let num_pit1_nodes = ctx.decode::<u16>("num_pit1_nodes")?;

        ctx.pad("reserved2", 2)?;

        let split0_node = ctx.decode::<u32>("split0_node")? as usize;
        let split1_node = ctx.decode::<u32>("split1_node")? as usize;
        let split2_node = ctx.decode::<u32>("split2_node")? as usize;
        let split3_node = ctx.decode::<u32>("split3_node")? as usize;

        let pole_position = ctx.decode::<SrPolePosition>("pole_position")?;

        let main_nodes: Vec<_> = (0..num_main_nodes)
            .map(|_| ctx.decode::<SrNode>("main_node"))
            .collect::<Result<Vec<_>, _>>()?;

        let pit0_nodes: Vec<_> = (0..num_pit0_nodes)
            .map(|_| ctx.decode::<SrNode>("pit0_node"))
            .collect::<Result<Vec<_>, _>>()?;

        let pit1_nodes: Vec<_> = (0..num_pit1_nodes)
            .map(|_| ctx.decode::<SrNode>("pit1_node"))
            .collect::<Result<Vec<_>, _>>()?;

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
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        if self.revision > 252 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 252,
                found: self.revision as usize,
            }
            .context("SRPATH unsupported revision"));
        }
        if self.main_nodes.len() > (u16::MAX as usize) {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: u16::MAX as usize,
                found: self.main_nodes.len(),
            }
            .context("SRPATH too many main nodes"));
        }
        if self.pit0_nodes.len() > (u16::MAX as usize) {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: u16::MAX as usize,
                found: self.pit0_nodes.len(),
            }
            .context("SRPATH too many pit0 nodes"));
        }
        if self.pit1_nodes.len() > (u16::MAX as usize) {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: u16::MAX as usize,
                found: self.pit1_nodes.len(),
            }
            .context("SRPATH too many pit1 nodes"));
        }

        ctx.encode("revision", &self.revision)?;
        ctx.encode("flags", &self.flags)?;
        ctx.encode("mini_rev", &self.mini_rev)?;
        ctx.pad("reserved", 3)?;

        ctx.encode("num_main_nodes", &(self.main_nodes.len() as u16))?;
        ctx.encode("num_pit0_nodes", &(self.pit0_nodes.len() as u16))?;
        ctx.encode("num_pit1_nodes", &(self.pit1_nodes.len() as u16))?;
        ctx.pad("reserved2", 2)?;

        ctx.encode("split0_node", &(self.split0_node as u32))?;
        ctx.encode("split1_node", &(self.split1_node as u32))?;
        ctx.encode("split2_node", &(self.split2_node as u32))?;
        ctx.encode("split3_node", &(self.split3_node as u32))?;

        ctx.encode("pole_position", &self.pole_position)?;

        for i in self
            .main_nodes
            .iter()
            .chain(self.pit0_nodes.iter())
            .chain(self.pit1_nodes.iter())
        {
            ctx.encode("node", i)?;
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
