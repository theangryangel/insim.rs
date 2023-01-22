//! Error types for the library.

use crate::protocol::relay::{RelayError, RelayErrorKind};
use deku::DekuError;
use miette::Diagnostic;
use std::io::ErrorKind;
use thiserror::Error as ThisError;

use super::trait;

// FIXME - we should probably drop the derive clone here?

#[non_exhaustive]
#[derive(ThisError, Diagnostic, Debug, Clone)]
pub enum Error {
    #[error("Unimplemented command or action")]
    Unimplemented,

    #[error("Shutdown")]
    Shutdown,

    #[error("Disconnected")]
    Disconnected,

    #[error("Maximum number of retries reached")]
    MaxConnectionAttempts,

    #[error("Unsupported Insim version: received {0:?}")]
    IncompatibleVersion(u8),

    #[error("IO error occurred: {kind}: {message}")]
    IO { kind: ErrorKind, message: String },

    #[error("Insim Relay error: {0:?}")]
    Relay(RelayErrorKind),

    #[error("Failed to decode packet: {0:?}")]
    Decoding(#[from] trait::DecodeError),

    #[error("Failed to encode packet: {0:?}")]
    Encoding(#[from] trait::EncodeError),

}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO {
            kind: e.kind(),
            message: e.to_string(),
        }
    }
}

impl From<RelayError> for Error {
    fn from(e: RelayError) -> Self {
        Error::Relay(e.err)
    }
}
