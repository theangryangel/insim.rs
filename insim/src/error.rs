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
    #[error("IO error occurred: {kind:?} {msg:?}")]
    IO {
        kind: std::io::ErrorKind,
        msg: String,
    },

    /// A timeout occurred whilst waiting for an operation
    #[error("Timeout: {0:?}")]
    Timeout(String),

    /// Failure to parse an address into SocketAddr
    #[error("Failed to parse address: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),

    /// Websocket IO error. Only applicable during a LFS World Relay connection
    #[error("Websocket Error: {0}")]
    WebsocketIO(String),

    /// Certain operations only allow for mods, this error indicates a standard vehicle was passed
    /// instead.
    #[error("Only Mods are permitted")]
    VehicleNotAMod,

    /// Certain operations only allow for standard vehicles, this error indicates a mod was passed
    /// instead.
    #[error("Only Standard vehicles are permitted")]
    VehicleNotStandard,

    // FIXME: Rename to ReadWriteBuf
    /// Placeholder
    #[error("Insim core error. Placeholder")]
    Core(insim_core::Error),
}

#[cfg(feature = "tokio")]
impl From<tokio::time::error::Elapsed> for Error {
    fn from(value: tokio::time::error::Elapsed) -> Self {
        Error::Timeout(value.to_string())
    }
}

impl From<insim_core::Error> for Error {
    fn from(value: insim_core::Error) -> Self {
        Error::Core(value) // FIXME
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

#[cfg(feature = "websocket")]
impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(value: tokio_tungstenite::tungstenite::Error) -> Self {
        // TODO a lot of this is less than ideal mapping
        // Do some research on better ways to handle this
        match value {
            tokio_tungstenite::tungstenite::Error::ConnectionClosed => Error::Disconnected,
            tokio_tungstenite::tungstenite::Error::AlreadyClosed => Error::Disconnected,
            tokio_tungstenite::tungstenite::Error::Utf8 => {
                Error::WebsocketIO("UTF-8 encoding error".into())
            },
            _ => Error::WebsocketIO(value.to_string()),
        }
    }
}
