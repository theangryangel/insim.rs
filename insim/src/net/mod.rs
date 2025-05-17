//! Network implementation

/// Async implementation
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tokio_impl;

/// Sync or blocking implementation
#[cfg(feature = "blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
pub mod blocking_impl;

pub(crate) mod codec;
pub(crate) mod mode;

pub use codec::Codec;
pub use mode::Mode;

/// If no data is received within this period of seconds, consider the Insim connection to be lost.
pub const DEFAULT_TIMEOUT_SECS: u64 = 70;
