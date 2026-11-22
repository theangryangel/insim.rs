//! Crate-wide error type.

/// Errors from a [`Host`](crate::Host).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AppError {
    /// The connection failed.
    #[error(transparent)]
    Insim(#[from] insim::Error),

    /// The host has stopped, so services can no longer be started or stopped.
    #[error("host closed")]
    Closed,
}
