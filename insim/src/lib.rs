#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(test, deny(warnings))]
#![cfg_attr(test, deny(unreachable_pub))]

use std::net::SocketAddr;

#[doc(hidden)]
pub mod builder;
#[doc(hidden)]
pub mod error;
pub mod identifiers;
pub mod insim;
pub mod net;
#[doc(hidden)]
pub mod packet;
pub mod relay;
#[doc(hidden)]
pub mod result;

/// The Insim Protocol Version Number supported by this library
pub const VERSION: u8 = 9;
/// The LFS World Relay address and port
pub const LFSW_RELAY_ADDR: &str = "isrelay.lfs.net:47474";

/// Rexport insim_core
pub use insim_core as core;

#[cfg(feature = "pth")]
/// Report insim_pth when pth feature is enabled
pub use insim_pth as pth;

#[cfg(feature = "smx")]
/// Report insim_smx when smx feature is enabled
pub use insim_smx as smx;

pub use builder::Builder;
pub use error::Error;
pub use packet::Packet;
pub use result::Result;

/// Shortcut method to create a TCP connection
///
/// # Examples
/// ```rust
/// let conn = insim::tcp("127.0.0.1:29999").connect().await?;
/// loop {
///     let packet = conn.read().await?;
///     println!("{:?}", packet);
/// }
/// ```
pub fn tcp<R: Into<SocketAddr>>(remote_addr: R) -> builder::Builder {
    builder::Builder::default().tcp(remote_addr)
}

/// Shortcut method to create a UDP connection.
/// If local_addr is not provided then we will bind to "0.0.0.0:0" (all addresses, random port).
///
/// # Examples
/// ```rust
/// let conn = insim::udp("127.0.0.1:29999", None).connect().await?;
/// loop {
///     let packet = conn.read().await?;
///     println!("{:?}", packet);
/// }
/// ```
pub fn udp<L: Into<Option<SocketAddr>>, R: Into<SocketAddr>>(
    remote_addr: R,
    local_addr: L,
) -> builder::Builder {
    builder::Builder::default().udp(remote_addr, local_addr)
}

/// Shortcut method to create a LFS World Relay connection.
///
/// # Examples
/// ```rust
/// let conn = insim::relay()
///     .relay_select_host("Nubbins AU Demo")
///     .relay_websocket(true)
///     .connect()
///     .await?;
/// loop {
///     let packet = conn.read().await?;
///     println!("{:?}", packet);
/// }
/// ```
pub fn relay() -> builder::Builder {
    builder::Builder::default().relay()
}
