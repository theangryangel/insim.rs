//! # insim_lyt
//!
//! Parse a Live for Speed lyt (layout) file.
//!
//! Supports only LFS 0.8+.
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

use bytes::{Bytes, BytesMut};

pub mod error;

pub use error::Error;
use insim_core::{DecodeContext, EncodeContext, object::ObjectInfo};

#[derive(Debug)]
/// LYT file
pub struct Lyt {
    /// Version
    pub version: u8,
    /// Revision
    pub revision: u8,
    /// Laps
    pub laps: u8,
    /// Mini Revision
    pub mini_rev: u8,
    /// Objects
    pub objects: Vec<ObjectInfo>,
}

impl Lyt {
    // XXX: Most pth files are only around 24KB, so we're going to pre-allocate at least that
    // amount of space.
    const DEFAULT_CAPACITY: usize = 24 * 1024;

    /// Read and parse a LYT file into a [Lyt] struct.
    pub fn read<R: Read>(mut reader: R) -> Result<Self, Error> {
        // Read the common header, MAGIC + version + revision
        let mut header = [0u8; 8];
        reader.read_exact(&mut header)?;
        let magic = &header[0..6];
        let version = header[6];
        let revision = header[7];

        // XXX: Reading into memory using read_to_end should be fine for the small files we're working with
        // here and we avoid any memory mapping, etc.
        match (magic, version) {
            (b"LFSLYT", 0) if revision <= 252 => {
                let mut data = Vec::with_capacity(Self::DEFAULT_CAPACITY);
                let _ = reader.read_to_end(&mut data)?;
                let mut buf = Bytes::from(data);
                let mut ctx = DecodeContext::new(&mut buf);
                let numo = ctx.decode::<u16>("numo")?;
                let laps = ctx.decode::<u8>("laps")?;
                let mini_rev = ctx.decode::<u8>("mini_rev")?;
                if mini_rev < 9 {
                    return Err(Error::UnsupportedMiniRev { mini_rev });
                }
                let mut objects = Vec::with_capacity(numo as usize);
                for _ in 0..numo {
                    objects.push(ctx.decode::<ObjectInfo>("objects")?);
                }
                Ok(Self {
                    version,
                    revision,
                    laps,
                    mini_rev,
                    objects,
                })
            },
            _ => Err(Error::UnsupportedVersion {
                magic: magic.to_vec(),
                version,
                revision,
            }),
        }
    }

    /// Write a file
    pub fn write<W: Write>(&self, mut writer: W) -> Result<usize, Error> {
        let mut written: usize = 0;
        let mut buf = BytesMut::new();
        let mut ctx = EncodeContext::new(&mut buf);
        ctx.encode("version", &self.version)?;
        ctx.encode("revision", &self.revision)?;
        let numo = self.objects.len();
        match TryInto::<u16>::try_into(numo) {
            Ok(numo) => ctx.encode("numo", &numo)?,
            Err(_) => {
                unimplemented!()
            },
        }
        ctx.encode("laps", &self.laps)?;
        ctx.encode("mini_rev", &self.mini_rev)?;
        for object in self.objects.iter() {
            ctx.encode("objects", object)?;
        }
        written += writer.write(b"LFSLYT")?;
        written += writer.write(&buf[..])?;
        Ok(written)
    }

    /// Read and parse a LYT file into a [Lyt] struct.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = fs::File::open(path)?;
        Self::read(file)
    }
}
