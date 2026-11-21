# Introduction and assumptions

This book introduces the `insim` crate. It assumes a basic understanding of Rust
and the InSim protocol, including connections (UCID), players (PLID), packets and
request identifiers.

The crate provides typed packets, protocol handling and networking for Live for
Speed. Packet and flag names follow InSim.txt where practical, dropping prefixes
such as `IS_` and `ISF_`. Some packed wire fields are split into separate Rust
fields for convenience.

The API uses Rust types for protocol concepts:

- `Duration` for packet intervals and other time values.
- Bitflags for options such as [IsiFlags](crate::insim::IsiFlags).
- [ConnectionId](crate::identifiers::ConnectionId) and
  [PlayerId](crate::identifiers::PlayerId) to distinguish connections from players.
- [RaceLaps](crate::insim::RaceLaps) for lap counts and timed races.
- [ObjectInfo](crate::insim::ObjectInfo) for layout objects.

## Tokio, blocking and sans-I/O

The `tokio` and `blocking` features are both enabled by default. Disable default
features and enable only the runtime you need when appropriate.

Your socket selects the session's runtime:

- Tokio sockets produce `Session<Tokio>`. Waiting operations such as `handshake`,
  `read`, `write`, `flush` and `shutdown` are async. The application supplies the
  Tokio runtime.
- Standard-library sockets produce `Session<Blocking>`. The same operations block
  the calling thread. Internally the sockets are nonblocking; the blocking adapter
  waits for readiness across the control socket and attached sockets.
- Operations that do not wait, such as `queue`, `try_read` and `attach`, have the
  same calling style for both runtimes.

Tokio examples below belong inside an async function. Network examples are
compile-checked as `no_run` doctests, so testing the documentation does not require
LFS. Examples that take an existing `connection` assume a mutable Tokio session.

For applications that provide their own I/O,
[InsimCodec](crate::net::InsimCodec) translates framed bytes and packets, while
[Context](crate::net::Context) handles the conversation using time supplied by the
caller. Neither component owns a socket or reads the clock.

## Establishing a connection

Create a socket, pass it to [Session](crate::Session), then start InSim with an
[Isi](crate::insim::Isi) handshake. Here, "session" means a link to LFS, rather than
a race session.

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example() -> insim::Result<()> {
use insim::{Session, identifiers::RequestId, insim::Isi};
use tokio::net::TcpStream;

let stream = TcpStream::connect("127.0.0.1:29999").await?;
let isi = Isi {
    reqi: RequestId(1),
    ..Isi::default()
};
let mut connection = Session::tcp(stream)?.handshake(isi).await?;

loop {
    let packet = connection.read().await?;
    println!("{packet:?}");
}
# }
```

`Session::tcp` and `Session::udp` initially return
[Unstarted](crate::Unstarted). `handshake` sends `Isi`, waits for a matching `Ver`,
and checks the protocol version. `Isi.reqi` must be nonzero, because zero does not
request that reply. The `Ver` packet remains available through `read()`.

For UDP control, connect the socket to LFS first:

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example() -> insim::Result<()> {
use insim::{Session, identifiers::RequestId, insim::Isi};
use tokio::net::UdpSocket;

let socket = UdpSocket::bind("0.0.0.0:0").await?;
socket.connect("127.0.0.1:29999").await?;
let isi = Isi {
    reqi: RequestId(1),
    udpport: socket.local_addr()?.port(),
    ..Isi::default()
};
let mut connection = Session::udp(socket)?.handshake(isi).await?;
let packet = connection.read().await?;
println!("{packet:?}");
# Ok(())
# }
```

The session does not rewrite `Isi.udpport`. Set it to the port LFS should send to;
zero requests replies at the UDP control socket's source port. UDP handshakes
retry initialization until a matching reply arrives or the handshake times out.

The blocking equivalent uses standard-library sockets:

```rust,no_run
# #[cfg(feature = "blocking")]
# fn example() -> insim::Result<()> {
use insim::{Session, identifiers::RequestId, insim::Isi};
use std::net::TcpStream;

let stream = TcpStream::connect("127.0.0.1:29999")?;
let isi = Isi {
    reqi: RequestId(1),
    ..Isi::default()
};
let mut connection = Session::tcp(stream)?.handshake(isi)?;
let packet = connection.read()?;
println!("{packet:?}");
# Ok(())
# }
```

If the caller has already initialized the link, `Session::tcp(socket)?.resume()`
or `Session::udp(socket)?.resume()` skips the handshake. Before choosing handshake
or resume, `Unstarted` can receive attached telemetry but cannot send application
InSim packets.

The older `insim::tcp`, `insim::udp` and `builder::Builder` APIs are deprecated.
Use explicit sockets and a session for new code.

## Driving the session and maintaining the connection

A session has no background task. Keep driving `read()` or `flush()` so incoming
packets, automatic keepalive replies and deadlines make progress. `write()` also
drives the session while flushing its output. A slow handler inside the read loop
can stall that progress; periodically entering the loop is not a guarantee that
keepalives will be handled in time.

Once InSim is active, its inactivity deadline is 70 seconds without a successfully
processed InSim packet. InSim packets on either the control socket or an attached
socket refresh that deadline. OutGauge and OutSim do not. Standalone telemetry
listeners and unstarted conversations have no inactivity timeout.

### Queueing, flushing and cancellation

`queue(packet)` encodes and accepts output without waiting. `flush()` drives both
reads and writes until the socket has accepted all queued output:

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example(connection: &mut insim::Session<insim::net::Tokio>) -> insim::Result<()> {
use insim::{WithRequestId, insim::TinyType};

connection.queue(TinyType::Ver.with_request_id(1))?;
connection.queue(TinyType::Sst.with_request_id(2))?;
connection.flush().await?;
# Ok(())
# }
```

`write(packet).await` queues once and then flushes. Cancelling `read`, `write` or
`flush` preserves accepted output and partial I/O progress. A cancelled write may
already have been sent or remain queued: call `flush()` to finish pending output.
Repeating `write(packet)` can send a duplicate.

Socket acceptance is not acknowledgement from LFS. Output still requires the
session to be driven, and dropping the session drops its pending output. The
owning `handshake` future is different from a borrowed read or write: cancelling
it drops the link.

Use `shutdown()` to send `TINY_CLOSE`, drain output and close the TCP write
direction. For Tokio, await it; for blocking sessions, call it directly.

## Constructing packets

[Packet](crate::Packet) identifies incoming InSim and attached telemetry packets.
Its payload types implement `Into<Packet>`. InSim subtypes such as
[TinyType](crate::insim::TinyType) can also be converted directly.

These three forms send the same InSim packet:

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example(connection: &mut insim::Session<insim::net::Tokio>) -> insim::Result<()> {
use insim::{Packet, insim::{Tiny, TinyType}};

connection.write(Packet::Tiny(Tiny {
    subt: TinyType::Ver,
    ..Tiny::default()
})).await?;

connection.write(Tiny {
    subt: TinyType::Ver,
    ..Tiny::default()
}).await?;

connection.write(TinyType::Ver).await?;
# Ok(())
# }
```

For requests that support replies, set a nonzero
[RequestId](crate::identifiers::RequestId). LFS echoes it in the response. This
correlates a request with its replies; it does not turn every instruction into a
request or suppress unsolicited packets.

The [WithRequestId](crate::WithRequestId) trait avoids constructing a full packet:

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example(connection: &mut insim::Session<insim::net::Tokio>) -> insim::Result<()> {
use insim::{Packet, WithRequestId, identifiers::RequestId, insim::{Tiny, TinyType}};

connection.write(Packet::Tiny(Tiny {
    reqi: RequestId(1),
    subt: TinyType::Ver,
})).await?;

connection.write(Tiny {
    reqi: RequestId(1),
    subt: TinyType::Ver,
}).await?;

connection.write(TinyType::Ver.with_request_id(1)).await?;
# Ok(())
# }
```

Attached OutGauge and OutSim variants are receive-only. Converting them to
`Packet` does not make them valid outgoing InSim packets.

# Cookbook

Working applications are available in the repository's
[examples directory](https://github.com/theangryangel/insim.rs/tree/main/examples),
including connection basics, telemetry, layout editing, in-game UI and PTH tools.

## Error handling

Session operations return `Result<_, insim::Error>`. Connection failures, version
mismatches, InSim timeouts and incoming-buffer overflow are terminal.

Complete UDP packets are returned in order. If a later packet is malformed, the
session logs a warning and discards only the malformed remainder. It continues
with other input without failing `read`, `flush`, `write` or `handshake` for that
decode error. Standalone listeners and low-level transports still report their
UDP decoding errors to their callers.

TCP framing/decoding errors and socket I/O errors remain terminal session errors.

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example(connection: &mut insim::Session<insim::net::Tokio>) -> insim::Result<()> {
use insim::Error;

loop {
    match connection.read().await {
        Ok(packet) => println!("{packet:?}"),
        Err(Error::Disconnected | Error::Timeout(_)) => break,
        Err(error) => return Err(error),
    }
}
# Ok(())
# }
```

`Error::Encode` means an outgoing packet could not be encoded. That packet was not
accepted, previously queued output remains intact, and the connection can still
be used. `Error::OutgoingOverflow` returned by application `queue()` means there
is no room for another packet; flush pending output before retrying.

While `flush` or `handshake` drives incoming traffic, the session buffers up to
16384 incoming packets. When the buffer is full it stops reading until you call
`read`: unread input waits in the operating system's socket buffers, TCP flow
control slows LFS, and the inactivity timeout is held. On UDP the operating
system may drop datagrams once its own buffer fills. Exhausting the reserved
automatic-response output capacity also closes the session.

## Receiving telemetry (MCI)

Enable [IsiFlags::MCI](crate::insim::IsiFlags::MCI) and a nonzero interval in the
handshake. Each [Mci](crate::insim::Mci) packet contains a batch of
[CompCar](crate::insim::CompCar) entries; an update can span multiple packets.
Coordinates are stored in fixed-point units, so use metre conversion helpers for
display:

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example() -> insim::Result<()> {
use insim::{Session, identifiers::RequestId, insim::{Isi, IsiFlags}};
use std::time::Duration;
use tokio::net::TcpStream;

let stream = TcpStream::connect("127.0.0.1:29999").await?;
let isi = Isi {
    reqi: RequestId(1),
    flags: IsiFlags::MCI,
    interval: Duration::from_millis(200),
    ..Isi::default()
};
let mut connection = Session::tcp(stream)?.handshake(isi).await?;

loop {
    if let insim::Packet::Mci(mci) = connection.read().await? {
        for car in &mci.info {
            let (x, y, z) = car.xyz.xyz_metres();
            println!(
                "plid={} speed={:.1}m/s x={x:.1}m y={y:.1}m z={z:.1}m",
                car.plid,
                car.speed.to_metres_per_sec(),
            );
        }
    }
}
# }
```

## Mixed TCP and UDP

Keep control traffic on TCP and receive positional updates and telemetry on an
attached UDP socket. Bind the socket, put its actual local port in `Isi.udpport`,
and attach a decoder before starting the handshake. Using the actual port also
supports binding to port `0`.

[Demux](crate::net::Demux) allows several protocols to share the UDP socket. All
packets then arrive through one `read()` loop; the session handles InSim framing
and routes automatic responses through the control transport.

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example() -> insim::Result<()> {
use insim::{
    Packet, Session,
    identifiers::RequestId,
    insim::{Isi, IsiFlags, SmallType},
    net::{Demux, InsimCodec, OutGaugeCodec, OutSimCodec},
};
use std::time::Duration;
use tokio::net::{TcpStream, UdpSocket};

let stream = TcpStream::connect("127.0.0.1:29999").await?;
let udp = UdpSocket::bind("0.0.0.0:0").await?;
let isi = Isi {
    reqi: RequestId(1),
    udpport: udp.local_addr()?.port(),
    flags: IsiFlags::MCI,
    interval: Duration::from_secs(1),
    ..Isi::default()
};
let decoder = Demux::new()
    .with(OutGaugeCodec)
    .with(OutSimCodec::default())
    .with(InsimCodec);
let mut connection = Session::tcp(stream)?
    .attach_udp(udp, decoder)?
    .handshake(isi).await?;

// Start OutSim and OutGauge when they are not already configured in cfg.txt.
connection.write(SmallType::Ssp(Duration::from_secs(1))).await?;
connection.write(SmallType::Ssg(Duration::from_secs(1))).await?;

loop {
    match connection.read().await? {
        Packet::Mci(packet) => println!("MCI: {packet:?}"),
        Packet::Outgauge(packet) => println!("OutGauge: {packet:?}"),
        Packet::Outsim(packet) => println!("OutSim: {packet:?}"),
        packet => println!("InSim: {packet:?}"),
    }
}
# }
```

The stack tries decoders in order; the first decoder that claims a datagram wins.
`SMALL_SSG` and `SMALL_SSP` send OutGauge and OutSim to `Isi.udpport`, the same
port as MCI and NLP, so they cannot be given separate ports from InSim. OutGauge
and original OutSim have no protocol markers and are recognized by size: OutSim
is 64 or 68 bytes and OutGauge 92 or 96. MCI never has those sizes, but an NLP
for 10 or 15 players is 64 or 96 bytes. Put the telemetry decoders first and
[InsimCodec](crate::net::InsimCodec) last. Known issue: with this order, an NLP
for 10 or 15 players is decoded as OutSim or OutGauge, with no error. Telemetry configured in LFS's
`cfg.txt` can instead be sent to a separate port with its own listener.
A decoder that claims malformed input returns an error instead of falling through
to another decoder. The session logs and skips that UDP input. Packets from different sockets have no common arrival ordering,
and `read()` does not currently report which socket supplied them.

All attached sockets must use the same runtime. `Isi.udpport` is passed unchanged
to LFS, allowing NAT or port forwarding. A port without a matching local UDP
socket produces a warning at handshake rather than an error.

The complete mixed-protocol example is in `examples/ssg-manual/`.

### Standalone and attached listeners

OutGauge and OutSim can also be received without an InSim session. Their listeners
have no inactivity timeout. `try_read()` returns a packet if available without
waiting; `read()` waits using the socket's runtime.

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example() -> insim::Result<()> {
use tokio::net::UdpSocket;

let socket = UdpSocket::bind("0.0.0.0:30000").await?;
let mut gauges = outgauge::Listener::udp(socket)?;
let gauge: outgauge::Outgauge = gauges.read().await?;
println!("{gauge:?}");

let socket = UdpSocket::bind("0.0.0.0:30001").await?;
let mut positions = outsim::Listener::udp(socket)?;
let position: outsim::OutsimPack = positions.read().await?;
println!("{position:?}");
# Ok(())
# }
```

Enable the corresponding `tokio` or `blocking` feature on the `outgauge` or
`outsim` dependency when calling its runtime-specific `read()` method.

OutSim's format is an explicit choice. The default codec and `Listener::udp`
return the original `OutsimPack`. For the extended format, configure LFS's
`OutSim Opts` in `cfg.txt` and supply the same options to the decoder or listener:

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example() -> insim::Result<()> {
use outsim::outsimpack2::{OutSimOpts, OutsimPack2};
use tokio::net::UdpSocket;

let opts = OutSimOpts::all(); // must match OutSim Opts in LFS's cfg.txt
let socket = UdpSocket::bind("0.0.0.0:30002").await?;
let mut positions = outsim::Listener::extended(socket, opts)?;
let position: OutsimPack2 = positions.read().await?;
println!("{position:?}");
# Ok(())
# }
```

For an attached decoder stack, use `OutSimCodec::extended(opts)` instead of
`OutSimCodec::default()`. The extended packet reaches a session as
`Packet::Outsim2`; the original format reaches it as `Packet::Outsim`.
`SMALL_SSP` controls the update interval; it does not select extended fields.

A standalone listener can be consumed by `attach`:

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example() -> insim::Result<()> {
use insim::{Session, identifiers::RequestId, insim::Isi};
use tokio::net::{TcpStream, UdpSocket};

let stream = TcpStream::connect("127.0.0.1:29999").await?;
// This example receives OutGauge configured in LFS to send to port 30000.
let socket = UdpSocket::bind("0.0.0.0:30000").await?;
let gauges = outgauge::Listener::udp(socket)?;
let isi = Isi { reqi: RequestId(1), ..Isi::default() };
let mut connection = Session::tcp(stream)?
    .attach(gauges)
    .handshake(isi).await?;
let packet = connection.read().await?;
println!("{packet:?}");
# Ok(())
# }
```

## Tracking player join / leave

Connections ([`crate::insim::Ncn`]) and players ([`crate::insim::Npl`]) are separate
events. A connection must exist before a player can, but a connection can outlast its
player (e.g. after pitting to spectate). To get an initial snapshot of all current
connections and players, send [`crate::insim::TinyType::Ncn`] and
[`crate::insim::TinyType::Npl`] after connecting:

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example(connection: &mut insim::Session<insim::net::Tokio>) -> insim::Result<()> {
use insim::{WithRequestId, insim::TinyType};

// request current state - replies arrive as normal Ncn/Npl packets with reqi echoed
connection.write(TinyType::Ncn.with_request_id(1)).await?;
connection.write(TinyType::Npl.with_request_id(1)).await?;

loop {
    match connection.read().await? {
        insim::Packet::Ncn(ncn) => println!("connection joined: ucid={}", ncn.ucid),
        insim::Packet::Cnl(cnl) => println!("connection left:   ucid={}", cnl.ucid),
        insim::Packet::Npl(npl) => println!("player joined:     plid={} ucid={}", npl.plid, npl.ucid),
        insim::Packet::Pll(pll) => println!("player left:       plid={}", pll.plid),
        _ => {},
    }
}
# Ok(())
# }
```

## Sending messages

Use [`crate::insim::Mst`] for chat text or slash commands, [`crate::insim::Msx`]
for longer chat text, and [`crate::insim::Mtc`] for a private message from the host.
`Mst` has a 64-byte text field; `Msx` has a 96-byte field. One byte is reserved for
termination, leaving up to 63 and 95 encoded text bytes respectively. Limits are
in encoded bytes, including codepage and colour sequences, rather than Rust
character counts.

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example(connection: &mut insim::Session<insim::net::Tokio>) -> insim::Result<()> {
use insim::insim::{Mst, Msx, Mtc};
use insim::identifiers::ConnectionId;

// Send chat text or a command using the shorter Mst field.
connection.write(Mst { msg: "Hello track!".into(), ..Default::default() }).await?;

connection.write(Msx { msg: "A longer chat message".into(), ..Default::default() }).await?;

// Private message to one connection (requires host privileges).
connection.write(Mtc {
    ucid: ConnectionId(3),
    text: "Hello driver!".into(),
    ..Default::default()
}).await?;
# Ok(())
# }
```

## Colouring strings

`insim` re-exports string helpers from the internal core crate for convenience, allowing
you to build coloured strings without looking up numeric colour codes.

```rust
use insim::Colour;

let hello = "Hello".red();
let world = String::from("World");
let combined = format!("{} {}", hello, world.blue());
```

## Escaping & unescaping Strings

`insim` re-exports string helpers from the internal core crate for convenience, allowing
you to quickly escape and unescape strings.

`insim` takes the opinionated approach that you should explicitly escape and unescape
strings. Originally this was automatic, however it became quite clear, quite quickly
that you paint yourself into a corner.

```rust
use insim::Escape;

let escaped = "^|*".escape();
let unescaped = escaped.unescape();
```

### Recommended processing order for incoming text

LFS text can contain codepages, colours, and escape sequences at the same time.
The safest order is:

1. Decode codepages first (if you're using the insim crate, not insim_core this happens
   automatically).
2. Perform colour-aware operations while the string is still escaped.
3. Unescape last.

This matters because unescaping is lossy with respect to control-marker intent.
For example, `^^0` unescapes to `^0`, which looks like a valid colour marker.
If you unescape first, later colour parsing can no longer tell whether that marker
was originally escaped text or a real colour control sequence.

## Layout objects

Layout objects use typed variants to represent their coordinates, colours and
other options.

You'll find most of it documented under [crate::insim::ObjectInfo].

### The `Axm` packet and `PmoAction`

The [Axm](crate::insim::Axm) packet carries a [PmoAction](crate::insim::PmoAction) enum
that unifies the action discriminant and its associated data into a single value. For
example, `PmoAction::AddObjects` carries the `Vec<ObjectInfo>` directly - there is no
separate `info` field. Cross-cutting flags (such as
[`PmoFlags::AVOID_CHECK`](crate::insim::PmoFlags)) live on [`Axm::flags`](crate::insim::Axm::flags)
rather than inside the variant.

Two variants have non-standard wire layouts and are handled transparently:

- **`PmoAction::Position`** - reports the editor cursor position when the user presses
  `O` with nothing selected. Only `xyz` and `heading` are meaningful; no object is added.
- **`PmoAction::GetZ`** - request or reply for Z values at given X, Y positions. Each
  entry is a [GetZEntry](crate::insim::GetZEntry) with an `xyz` and an `adjusted` flag.
  Set `Zbyte` to `240` to request the highest point at that X, Y; on reply `adjusted`
  indicates whether the value was successfully adjusted.

A simple example adding a single tyre stack to the track:

```rust,no_run
# #[cfg(feature = "tokio")]
# async fn example(connection: &mut insim::Session<insim::net::Tokio>) -> insim::Result<()> {
use insim::insim::{Axm, PmoAction};
use insim::core::object::{ObjectInfo, ObjectCoordinate};
use insim::core::object::tyres::{Tyres, TyreColour};
use insim::core::heading::ObjectHeading;

connection.write(Axm {
    action: PmoAction::AddObjects(vec![
        ObjectInfo::TyreStack4(Tyres {
            xyz: ObjectCoordinate::new(0, 0, 0),
            colour: TyreColour::Blue,
            floating: false,
            heading: ObjectHeading::ZERO,
        })
    ]),
    ..Default::default()
}).await?;
# Ok(())
# }
```
