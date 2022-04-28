//! Error types for the library.

use crate::protocol::relay::ErrorType as RelayErrorType;
use std::io::Error as IoError;

// TODO: use thiserror to simplify this

#[derive(Debug)]
pub enum Error {
    /// Unimplemented command or action
    Unimplemented,

    /// Currently shutdown
    Shutdown,

    /// Currently disconnected
    Disconnected,

    /// Describes when a timeout occurs communicating with the Insim server.
    Timeout,

    /// Describes when the maximum number of retries is reached.
    MaxConnectionAttempts,

    /// Describes when Insim version is not supported.
    IncompatibleVersion,

    /// Wraps ::std::io::Error.
    IO(IoError),

    /// Describes when a given input is too large.
    TooLarge,

    /// Insim Relay Error
    RelayError(RelayErrorType),
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Error::IO(err)
    }
}
