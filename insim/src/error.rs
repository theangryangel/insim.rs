use bytes::Bytes;

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
/// Errors that may occur during an InSim connection.
///
/// Connection failures and protocol timeouts are terminal. Encoding errors and
/// application queue capacity errors leave the connection usable. Sessions log
/// and skip malformed UDP input; TCP decode errors and socket errors are returned.
pub enum Error {
    /// Invalid explicit handshake configuration.
    #[error("invalid handshake: {0}")]
    InvalidHandshake(&'static str),
    /// Transport framing failed. Sessions log and skip UDP framing/decoding errors;
    /// TCP framing errors remain terminal.
    #[error(transparent)]
    Transport(insim_core::net::Error),
    /// Outgoing storage reached its capacity; flush before queueing more.
    #[error("session outgoing buffer is full")]
    OutgoingOverflow,

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

    /// Partial decode. The packet was framed and decoded but left bytes unread,
    /// which usually means a packet definition does not match LFS. The connection
    /// remains usable.
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

impl From<insim_core::net::Error> for Error {
    fn from(error: insim_core::net::Error) -> Self {
        match error {
            insim_core::net::Error::Io(error) => Self::IO(error),
            insim_core::net::Error::Disconnected => Self::Disconnected,
            insim_core::net::Error::Encode(error) => Self::Encode(error),
            insim_core::net::Error::Decode {
                input,
                offset,
                error,
            } => Self::Decode {
                input,
                offset,
                error,
            },
            insim_core::net::Error::IncompleteDecode { input, remaining } => {
                Self::IncompleteDecode { input, remaining }
            },
            error => Self::Transport(error),
        }
    }
}
