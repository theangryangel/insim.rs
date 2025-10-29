//! Error and Result for Runtime

/// Error for Runtime
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Not in extension map
    #[error("Extension {0} not registered")]
    UnknownExtension(&'static str),

    /// Error in Stage
    #[error("Stage handler error: {0}")]
    StageHandler(Box<dyn std::error::Error + Send + Sync>),
}

/// Results for the runtime
pub type Result<T> = std::result::Result<T, Error>;
