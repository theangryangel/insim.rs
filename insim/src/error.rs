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

    /// A timeout occured whilst waiting for an operation
    #[error("Timeout: {0:?}")]
    Timeout(String),

    /// Failure to parse an address into SocketAddr
    #[error("Failed to parse address: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),

    #[error("Shutdown")]
    Shutdown,

    #[error("Websocket Error: {0}")]
    WebsocketIO(String),

    #[error("Insim Core error {0}")]
    BinRw(String),

    #[error("Only Mods are permitted")]
    VehicleNotAMod,

    #[error("Only Standard vehicles are permitted")]
    VehicleNotStandard,
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(value: tokio::time::error::Elapsed) -> Self {
        Error::Timeout(value.to_string())
    }
}

impl From<insim_core::binrw::Error> for Error {
    fn from(value: insim_core::binrw::Error) -> Self {
        Error::BinRw(value.to_string()) // FIXME
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

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(value: tokio_tungstenite::tungstenite::Error) -> Self {
        // TODO a lot of this is less than ideal mapping
        // Do some research on better ways to handle this
        match value {
            tokio_tungstenite::tungstenite::Error::ConnectionClosed => Error::Disconnected,
            tokio_tungstenite::tungstenite::Error::AlreadyClosed => Error::Disconnected,
            tokio_tungstenite::tungstenite::Error::Utf8 => {
                Error::WebsocketIO("UTF-8 encoding error".into())
            }
            _ => Error::WebsocketIO(value.to_string()),
        }
    }
}
