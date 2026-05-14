//! Crate-wide error type.

/// Errors produced by the `insim_app` runtime and surfaced to handlers/middleware/spawned tasks.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AppError {
    /// Underlying InSim connection error.
    #[error(transparent)]
    Insim(#[from] insim::Error),

    /// The dispatcher's back-channel is closed (the runtime is shutting down).
    #[error("dispatcher channel closed")]
    Closed,

    /// A user-supplied handler returned an error.
    #[error("handler: {0}")]
    Handler(#[source] Box<dyn std::error::Error + Send + Sync>),
}
