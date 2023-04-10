use crate::error::Error;

/// A specialized [`Result`] type for insim.
pub type Result<T> = std::result::Result<T, Error>;
