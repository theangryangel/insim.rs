//! [`HostError`]: errors from a [`Host`](crate::kitcar::Host).

/// Errors from a [`Host`](crate::kitcar::Host).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HostError {
    /// The connection failed.
    #[error(transparent)]
    Insim(#[from] insim::Error),

    /// The host has stopped.
    #[error("host closed")]
    Closed,
}
