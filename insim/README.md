A friendly, Rust idiomatic library for the InSim protocol used by [Live for Speed](https://www.lfs.net/) racing simulator.

The focus of this library is providing a high level, strongly typed primitives that are difficult to misuse and have reasonable performance, rather than be a thin layer over a series of bytes.

We prioritise compatibility with the latest LFS protocol specifications. Given LFS is now effectively in an evergreen release cycle, support for superseded protocol versions is systematically removed in new releases, which is a key measure for reducing technical debt and maintenance burden.

Where possible this crate aligns the naming of fields in packets to match the [original Insim specification](https://en.lfsmanual.net/wiki/InSim.txt).

In a handful of circumstances we have needed to rename, or separate some fields to align with the crate's key focus.

# Installation

`insim` is on crates.io and can be used by adding `insim` to your dependencies in your project’s Cargo.toml.
Or more simply, just run `cargo add insim`.

If you want to use an unreleased version you can also reference the GitHub repository.

# Key Features

- Async (tokio) and blocking support.
- insim over TCP or UDP.
  - Create sockets explicitly and start a `Session` with a handshake.
  - If you prefer to handle that yourself, use `net::InsimCodec` for translation and `net::Context` for protocol state.
  - Attach UDP sockets or telemetry listeners to a session; use `Demux` when protocols share a socket.
- Handles keepalive responses while the session is driven through reads or flushes.
- Packets are represented by the `Packet` enum for ease of handling.
  - Many of the types within a `Packet` variant implement `Into<Packet>`, allowing you to
    avoid complex and tedious variable construction.
  - With helper traits like `WithRequestId` also implemented on `Packet` and many types
    within `Packet`.
- String utilities cover colours (`Colour`) and escaping (`Escape`), plus codepage handling in the core crate
  automatically convert strings from LFS into utf8 and back again.
- Did I mention the library is strongly typed, prefering rust native types where
  possible?

# Quick Start

You can find a wide range of examples in the upstream repository under [examples/](https://github.com/theangryangel/insim.rs/tree/main/examples).

What you will find there:

- Connection basics (async and blocking TCP/UDP).
- Layout objects / AXM packet tooling.
- Live telemetry and in-game UI.
- Other protocols (Outsim and Outgauge).
- PTH tooling (PTH to SVG conversion).

and a more comprehensive guide within the [book / guide](_book).

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example() -> insim::Result<()> {
use insim::{Session, identifiers::RequestId, insim::{Isi, IsiFlags}};
use std::time::Duration;

let stream = tokio::net::TcpStream::connect("127.0.0.1:29999").await?;
let isi = Isi {
    reqi: RequestId(1),
    flags: IsiFlags::MCI,
    interval: Duration::from_secs(1),
    ..Isi::default()
};
let mut connection = Session::tcp(stream)?.handshake(isi).await?;
loop {
    let packet = connection.read().await?;
    println!("{packet:?}");
}
# }
```

# Book / Cookbook / Guide

A more comprehensive guide can be found within the [book / cookbook / guide](_book).

# Crate features

| Name                   | Description                            | Default? |
| ---------------------- | -------------------------------------- | -------- |
| `serde`                | Enable serde support                   | No       |
| `tokio`                | Enable tokio support                   | Yes      |
| `blocking`             | Enable blocking/sync support           | Yes      |
| `allow-unknown-packet` | Unknown packets do not return an error | No       |

# Release Notes / Migration

You'll always find the release notes and any breaking changes / migration notes at
<https://github.com/theangryangel/insim.rs/releases>.

# Related crates

You might also find these related crates useful:

- `insim_core` - contains shared low-level types and utilities. Most users should only depend on `insim`. This is re-exported as `core` from insim.
- `insim_pth` – for reading and writing LFS PTH and PIN files.
- `outgauge` - "sans-io" implementation of the LFS outgauge protocol.
- `outsim` - "sans-io" implementation of the LFS outsim protocol.
- `insim_extra` - an unreleased, unstable suite of micro libraries to help with common
  LFS tasks: presence, game state, advanced button/ui abstraction inspired by Phoenix
  Liveview.
- `kitcar` - an unreleased library designed to make mini-games for LFS quickly, with an
  API inspired by Axum and Bevy.

They follow the same design focus and can be found in the same GitHub repository, or on
crates.io.

## Sessions and attached telemetry

`Session` means a link to LFS. The socket selects the runtime; the application
owns the Tokio executor when using Tokio sockets.

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example() -> insim::Result<()> {
use insim::{Session, identifiers::RequestId, insim::Isi};
use insim::net::{Demux, InsimCodec, Listener, OutGaugeCodec};

let stream = tokio::net::TcpStream::connect("127.0.0.1:29999").await?;
let udp = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
let isi = Isi {
    reqi: RequestId(1), // required for a correlated Ver reply
    udpport: udp.local_addr()?.port(),
    ..Isi::default()
};
let mixed = Listener::udp(udp, Demux::new().with(OutGaugeCodec).with(InsimCodec))?;
let og_socket = tokio::net::UdpSocket::bind("0.0.0.0:30000").await?;
let gauge = outgauge::Listener::udp(og_socket)?;
let mut session = Session::tcp(stream)?
    .attach(mixed)
    .attach(gauge)
    .handshake(isi).await?;

// The handshake's Ver is retained for read().
loop {
    let packet = session.read().await?;
    println!("{packet:?}");
}
# }
```

Standard-library sockets produce `Session<Blocking>` with synchronous methods.
Standalone telemetry listeners have no inactivity timeout. Before initialization,
a session has no InSim deadline either. All sockets attached to a session must
belong to the same runtime. For a UDP
control connection, connect the socket to LFS before calling `Session::udp`.
`resume()` skips initialization for a link already initialized by the caller.

`queue` accepts a packet without waiting; `flush` drives reads and writes until
all pending output is accepted by the socket. Cancelling `read`, `write` or
`flush` preserves queued output and partial TCP progress. Retrying a cancelled
`write(packet)` can duplicate it: use `flush` to finish existing output.
Dropping the owning handshake future drops the link. Sessions have no background
task; protocol progress requires driving them.

Complete UDP packets are returned in order. The session logs and skips malformed
UDP input, discarding only its remainder. That decode error does not fail a read,
write, flush or handshake, undo packets already returned, or affect the next datagram.
Standalone listeners and low-level transports still report UDP decode errors.
Stream framing failures are terminal. OutGauge and OutSim never extend InSim's
inactivity deadline. All InSim packets, including those on attached listeners,
reach protocol processing and normal reads.

Decoder selection is ordered. OutGauge and legacy OutSim have no magic numbers
and are recognized by size, and `SMALL_SSG`/`SMALL_SSP` send them to the same
port as MCI and NLP. Put the telemetry decoders first and `InsimCodec` last.
Known issue: an NLP for 10 or 15 players has the same size as OutSim or OutGauge
and is decoded as telemetry. Extended OutSim without its header
is recognized only by its configured size. A decoder that claims a malformed datagram returns
an error rather than falling through to another format.

The session buffers up to 16384 incoming packets while operations such as
`flush` or `handshake` are driving input. When the buffer is full it stops
reading until you call `read`; unread input waits in the operating system's
socket buffers, TCP flow control slows LFS, and the inactivity timeout is held.
On UDP the operating system may drop datagrams once its own buffer fills. Outgoing application queueing is limited to approximately 1 MiB;
flush before queueing more. The session itself never drops or coalesces telemetry.
