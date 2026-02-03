#![doc = include_str!("../README.md")]
#![cfg_attr(test, deny(warnings, unreachable_pub))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[cfg(any(feature = "blocking", feature = "tokio"))]
use std::net::SocketAddr;

#[macro_use]
mod macros;

#[cfg(any(feature = "blocking", feature = "tokio"))]
pub mod address;
#[cfg(any(feature = "blocking", feature = "tokio"))]
pub mod builder;
#[doc(hidden)]
pub mod error;
pub mod identifiers;
pub mod insim;
pub mod net;
pub mod packet;
#[doc(hidden)]
pub mod result;

/// The Insim Protocol Version Number supported by this library
pub const VERSION: u8 = 10;

/// Why 255 * 4? Because the size of a packet is a u8, with a max byte size of 255.
/// In "compressed" mode the raw size is multiplied by 4.
pub(crate) const MAX_SIZE_PACKET: usize = (u8::MAX as usize) * 4;

pub(crate) const DEFAULT_BUFFER_CAPACITY: usize = MAX_SIZE_PACKET * 6;

pub use error::Error;
/// Rexport insim_core
pub use insim_core as core;
pub use packet::{Packet, WithRequestId};
pub use result::Result;
pub use core::string::{colours::Colour, escaping::Escape};

/// Shortcut method to create a TCP connection
///
/// # Examples
///
/// Supports both blocking and tokio. Swap about `connect_async` for `connect` and remove the
/// `.await` annotations.
///
/// ```rust
/// let conn = insim::tcp("127.0.0.1:29999").connect_async().await?;
/// loop {
///     let packet = conn.read().await?;
///     println!("{:?}", packet);
/// }
/// ```
#[cfg(any(feature = "blocking", feature = "tokio"))]
pub fn tcp<R: Into<address::Addr>>(remote_addr: R) -> builder::Builder {
    builder::Builder::default().tcp(remote_addr)
}

/// Shortcut method to create a UDP connection.
/// If local_addr is not provided then we will bind to "0.0.0.0:0" (all addresses, random port).
///
/// # Examples
///
/// Supports both blocking and tokio. Swap about `connect_async` for `connect` and remove the
/// `.await` annotations.
///
/// ```rust
/// let conn = insim::udp("127.0.0.1:29999", None).connect_async().await?;
/// loop {
///     let packet = conn.read().await?;
///     println!("{:?}", packet);
/// }
/// ```
#[cfg(any(feature = "blocking", feature = "tokio"))]
pub fn udp<L: Into<Option<SocketAddr>>, R: Into<address::Addr>>(
    remote_addr: R,
    local_addr: L,
) -> builder::Builder {
    builder::Builder::default().udp(remote_addr, local_addr)
}
