//! Error types for the library.

use crate::protocol::relay::ErrorType as RelayErrorType;
use std::io::ErrorKind;
use thiserror::Error as ThisError;

// TODO: use thiserror to simplify this

#[derive(ThisError, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("Unimplemented command or action")]
    Unimplemented,

    #[error("Shutdown")]
    Shutdown,

    #[error("Disconnected")]
    Disconnected,

    #[error("Timeout when communicating with the Insim server")]
    Timeout,

    #[error("Maximum number of retries reached")]
    MaxConnectionAttempts,

    #[error("Unsupported Insim version")]
    IncompatibleVersion,

    #[error("IO error occurred")]
    IO { kind: ErrorKind, message: String },

    #[error("Input is too large")]
    TooLarge,

    #[error("Insim Relay error")]
    RelayError(RelayErrorType),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO {
            kind: e.kind(),
            message: e.to_string(),
        }
    }
}
