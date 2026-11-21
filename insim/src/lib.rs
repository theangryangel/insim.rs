#![doc = include_str!("../README.md")]
#![cfg_attr(test, deny(warnings, unreachable_pub))]

#[cfg(any(feature = "blocking", feature = "tokio"))]
use std::net::SocketAddr;

#[macro_use]
mod macros;

#[cfg(feature = "serde")]
pub mod duration_serde;

#[cfg(any(feature = "blocking", feature = "tokio"))]
/// Address types for connecting to LFS.
pub mod address;
#[cfg(any(feature = "blocking", feature = "tokio"))]
/// Builder for establishing an InSim connection.
pub mod builder;
/// Error type for InSim connection failures.
pub mod error;
pub mod identifiers;
pub mod insim;
pub mod net;
pub mod packet;
#[doc(hidden)]
pub mod result;

#[doc = include_str!("../docs/book.md")]
pub mod _book {}

/// The Insim Protocol Version Number supported by this library
pub const VERSION: u8 = 10;

/// Why 255 * 4? Because the size of a packet is a u8, with a max byte size of 255.
/// In "compressed" mode the raw size is multiplied by 4.
pub(crate) const MAX_SIZE_PACKET: usize = (u8::MAX as usize) * 4;

pub use core::string::{colours::Colour, escaping::Escape};

pub use error::Error;
/// Rexport insim_core
pub use insim_core as core;
pub use packet::{Packet, WithRequestId};
pub use result::Result;

/// Deprecated shortcut returning a TCP connection builder.
///
/// Prefer [`Session::tcp`] with an owned socket and an explicit handshake.
#[cfg(any(feature = "blocking", feature = "tokio"))]
#[deprecated(note = "Connect a TCP socket, then use Session::tcp(socket)?.handshake(isi)")]
#[allow(deprecated)]
pub fn tcp<R: Into<address::Addr>>(remote_addr: R) -> builder::Builder {
    builder::Builder::default().tcp(remote_addr)
}

/// Deprecated shortcut returning a UDP connection builder.
/// If `local_addr` is omitted, the builder binds to `0.0.0.0:0`.
///
/// Prefer [`Session::udp`] with an owned, connected socket and an explicit handshake.
#[cfg(any(feature = "blocking", feature = "tokio"))]
#[deprecated(note = "Bind and connect a UDP socket, then use Session::udp(socket)?.handshake(isi)")]
#[allow(deprecated)]
pub fn udp<L: Into<Option<SocketAddr>>, R: Into<address::Addr>>(
    remote_addr: R,
    local_addr: L,
) -> builder::Builder {
    builder::Builder::default().udp(remote_addr, local_addr)
}

pub use net::{Session, Unstarted};
