use bytes::Bytes;

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
/// The Errors that may occur during an Insim connection.
pub enum Error {
    /// Connection is disconnected
    #[error("Disconnected")]
    Disconnected,

    /// Connection has detected in incompatible version
    #[error("Unsupported Insim version: received {0:?}")]
    IncompatibleVersion(u8),

    /// IO Error, i.e. initial connection failed, etc.
    #[error("IO error occurred: {0}")]
    IO(#[from] std::io::Error),

    /// A timeout occurred whilst waiting for an operation
    #[error("Timeout: {0:?}")]
    Timeout(String),

    /// Failure to parse an address into SocketAddr
    #[error("Failed to parse address: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),

    /// Certain operations only allow for mods, this error indicates a standard vehicle was passed
    /// instead.
    #[error("Only Mods are permitted")]
    VehicleNotAMod,

    /// Certain operations only allow for standard vehicles, this error indicates a mod was passed
    /// instead.
    #[error("Only Standard vehicles are permitted")]
    VehicleNotStandard,

    /// Encode Error
    #[error("Encode error: {0}")]
    Encode(#[from] insim_core::EncodeError),

    /// Decode Error
    #[error("Decode error {error} at offset {offset}: {:?}", input.as_ref())]
    Decode {
        offset: usize,
        input: Bytes,
        error: insim_core::DecodeError,
    },

    /// Partial decode
    #[error("Partial decode. Likely invalid packet definition. Decoded {:?}, remaining {:?}", input.as_ref(), remaining.as_ref())]
    IncompleteDecode {
        /// original input
        input: Bytes,

        /// remaining
        remaining: Bytes,
    },
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl From<tokio::time::error::Elapsed> for Error {
    fn from(value: tokio::time::error::Elapsed) -> Self {
        Error::Timeout(value.to_string())
    }
}
