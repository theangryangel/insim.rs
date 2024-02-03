//! Network implementation

/// Async implementation
#[cfg(feature = "async")]
pub mod r#async;

/// Sync or blocking implementation
#[cfg(feature = "blocking")]
pub mod blocking;

pub(crate) mod codec;
pub(crate) mod mode;

pub use codec::Codec;
pub use mode::Mode;

/// If no data is received within this period of seconds, consider the Insim connection to be lost.
pub const DEFAULT_TIMEOUT_SECS: u64 = 90;
