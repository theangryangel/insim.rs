use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
    Timeout,
    MaxConnectionAttempts,
    IncompatibleVersion,
    IO(IoError),
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Error::IO(err)
    }
}
