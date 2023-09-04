use insim_core::{DecodableError, EncodableError};

#[non_exhaustive]
#[derive(thiserror::Error, Debug, Clone)]
/// A specialized [`Error`] type for insim.
pub enum Error {
    /// Connection is disconnected
    #[error("Disconnected")]
    Disconnected,

    /// Connection has detected in incompatible version
    #[error("Unsupported Insim version: received {0:?}")]
    IncompatibleVersion(u8),

    /// IO Error, i.e. initial connection failed, etc.
    #[error("IO error occurred: {kind:?} {msg:?}")]
    IO {
        kind: std::io::ErrorKind,
        msg: String,
    },

    /// LFS World Relay Error
    #[error("Insim Relay error: {0:?}")]
    Relay(#[from] crate::packets::relay::RelayError),

    /// Failed to decode a packet
    #[error("Failed to decode packet: {0:?}")]
    Decoding(#[from] DecodableError),

    /// Failed to encode a packet
    #[error("Failed to encode packet: {0:?}")]
    Encoding(#[from] EncodableError),

    /// A timeout occured whilst waiting for an operation
    #[error("Timeout: {0:?}")]
    Timeout(String),

    /// Failure to parse an address into SocketAddr
    #[error("Failed to parse address: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),

    #[error("Shutdown")]
    Shutdown,
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(value: tokio::time::error::Elapsed) -> Self {
        Error::Timeout(value.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IO {
            kind: value.kind(),
            msg: value.to_string(),
        }
    }
}
