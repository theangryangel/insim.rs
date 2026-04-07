use bytes::Bytes;

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
/// Errors that may occur during an InSim connection.
///
/// Most variants are fatal - once returned, the connection is no longer usable and you
/// should reconnect via [`crate::builder::Builder`]. The exceptions are [`Error::Encode`],
/// [`Error::VehicleNotAMod`], and [`Error::VehicleNotStandard`], which indicate a problem
/// with a packet you tried to send and leave the connection intact.
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
        /// Byte offset within the packet where decoding failed.
        offset: usize,
        /// The raw bytes of the packet that could not be decoded.
        input: Bytes,
        #[source]
        /// The underlying decode error.
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

    /// The background task spawned via [`crate::builder::Builder::spawn`] has exited.
    #[cfg(feature = "tokio")]
    #[error("Unable to send to the spawned insim connection. Task died?")]
    SpawnedDead,
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl From<tokio::time::error::Elapsed> for Error {
    fn from(value: tokio::time::error::Elapsed) -> Self {
        Error::Timeout(value.to_string())
    }
}
