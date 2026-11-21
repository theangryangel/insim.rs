//! Byte transport, runtime adapters and composable packet decoders.
//!
//! Decoders do not own sockets or clocks. Transports retain all I/O progress.

mod codec;
mod listener;
mod runtime;
mod transport;

pub use codec::{Decoder, Demux, Encoder, Error, Result};
pub use listener::Listener;
#[cfg(feature = "blocking")]
pub use runtime::Blocking;
#[cfg(feature = "tokio")]
pub use runtime::Tokio;
pub use runtime::{DatagramSocket, Interest, Runtime, Socket, StreamSocket};
pub use transport::{ReceiveTransport, Transport};
