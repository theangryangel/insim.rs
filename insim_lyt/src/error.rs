//! Error
use std::io::ErrorKind;

use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum Error {
    #[error(
        "Unsupported version or revision, found magic '{magic:?}', version '{version}', revision '{revision}'"
    )]
    UnsupportedVersion {
        magic: Vec<u8>,
        version: u8,
        revision: u8,
    },

    #[error("Unsupported mini revision '{mini_rev}'")]
    UnsupportedMiniRev { mini_rev: u8 },

    #[error("IO Error: {kind}: {message}")]
    IO { kind: ErrorKind, message: String },

    #[error("ReadWriteBuf Err {0:?}")]
    Encode(#[from] insim_core::EncodeError),

    #[error("ReadWriteBuf Err {0:?}")]
    Decode(#[from] insim_core::DecodeError),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO {
            kind: e.kind(),
            message: e.to_string(),
        }
    }
}
