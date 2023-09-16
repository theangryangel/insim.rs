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

    #[error("Websocket Error: {0}")]
    WebsocketIO(String),
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

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(value: tokio_tungstenite::tungstenite::Error) -> Self {
        // TODO a lot of this is less than ideal mapping
        // Do some research on better ways to handle this
        match value {
            tokio_tungstenite::tungstenite::Error::ConnectionClosed => Error::Disconnected,
            tokio_tungstenite::tungstenite::Error::AlreadyClosed => Error::Disconnected,
            tokio_tungstenite::tungstenite::Error::Io(inner) => inner.into(),
            tokio_tungstenite::tungstenite::Error::Tls(i) => Error::WebsocketIO(i.to_string()),
            tokio_tungstenite::tungstenite::Error::Capacity(i) => Error::WebsocketIO(i.to_string()),
            tokio_tungstenite::tungstenite::Error::Protocol(i) => Error::WebsocketIO(i.to_string()),
            tokio_tungstenite::tungstenite::Error::WriteBufferFull(i) => {
                Error::WebsocketIO(i.to_string())
            }
            tokio_tungstenite::tungstenite::Error::Utf8 => {
                Error::WebsocketIO("UTF-8 encoding error".into())
            }
            tokio_tungstenite::tungstenite::Error::Url(i) => Error::WebsocketIO(i.to_string()),
            tokio_tungstenite::tungstenite::Error::Http(i) => {
                Error::WebsocketIO(format!("{:?}", i))
            }
            tokio_tungstenite::tungstenite::Error::HttpFormat(i) => {
                Error::WebsocketIO(i.to_string())
            }
        }
    }
}
