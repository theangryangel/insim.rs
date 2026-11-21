//! Sessions, pure codecs and composable UDP listeners.
mod context;
mod insim_codec;
mod request;
mod session;
#[cfg(feature = "tokio")]
mod split;
mod tap;

pub use context::Context;
pub use insim_codec::InsimCodec;
#[cfg(feature = "blocking")]
pub use insim_core::net::Blocking;
#[cfg(feature = "tokio")]
pub use insim_core::net::Tokio;
pub use insim_core::net::{
    DatagramSocket, Decoder, Encoder, Interest, Listener, ReceiveTransport, Runtime, Socket,
    StreamSocket, Transport,
};
pub use outgauge::OutGaugeCodec;
pub use outsim::OutSimCodec;
pub use request::{DEFAULT_REQUEST_IDS, REQUEST_ALL_QUIET, Replies, Reply, Response};
pub use session::{Session, Unstarted};
#[cfg(feature = "tokio")]
pub use split::{Driver, Receiver, Sender};
pub use tap::{Policy, Tap};

/// InSim inactivity timeout. OutGauge and OutSim do not reset it.
pub const DEFAULT_TIMEOUT_SECS: u64 = 70;

/// Decoder stack producing the session's combined packet type.
///
/// **If the stack includes telemetry, put [`InsimCodec`] last.** OutGauge and
/// OutSim are recognized only by size (OutSim 64/68 bytes, OutGauge 92/96), while
/// `InsimCodec`'s check is loose enough to claim many telemetry datagrams.
///
/// Known issue: an NLP for 10 or 15 players is 64 or 96 bytes, so with this order
/// it is decoded as OutSim or OutGauge, with no error. MCI sizes never overlap.
///
/// ```rust
/// use insim::net::{Demux, InsimCodec, OutGaugeCodec, OutSimCodec};
///
/// let decoder = Demux::new()
///     .with(OutGaugeCodec)
///     .with(OutSimCodec::default())
///     .with(InsimCodec); // always last
/// ```
pub type Demux = insim_core::net::Demux<crate::Packet>;
