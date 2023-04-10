//! Error types for the library.

use insim_core::{DecodableError, EncodableError};

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Disconnected")]
    Disconnected,

    #[error("Unsupported Insim version: received {0:?}")]
    IncompatibleVersion(u8),

    #[error("IO error occurred: {0:?}")]
    IO(#[from] std::io::Error),

    #[cfg(feature = "relay")]
    #[error("Insim Relay error: {0:?}")]
    Relay(#[from] crate::packets::relay::RelayError),

    #[error("Failed to decode packet: {0:?}")]
    Decoding(#[from] DecodableError),

    #[error("Failed to encode packet: {0:?}")]
    Encoding(#[from] EncodableError),

    #[error("Timeout")]
    Timeout(String),

    #[error("Failed to parse address: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(value: tokio::time::error::Elapsed) -> Self {
        Error::Timeout(value.to_string())
    }
}
