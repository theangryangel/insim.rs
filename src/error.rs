use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
    Timeout,
    MaxConnectionAttempts,
    IncompatibleVersion,
    IO(IoError),
}
