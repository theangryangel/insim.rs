# Introduction & Assumptions

This book is intended as an introduction to the the insim crate.

The guide does assume basic understanding of both rust, and the insim protocol and
concepts and conventions within Live For Speed.

Fundamentally the insim crate is not just a thin wrapper on top of a series of bytes. It
actively tries to make things as easy for you as possible by providing abstractions and
opinionated standards.

## Compatibility with InSim.txt?

- Where ever possible we have attempted to ensure that packets, fields and flags named closely to the original insim spec
- There are some exceptions to this, such as thrbrk in the contact packet, which we have
  split for ergonomics.
- By convention we have generally dropped the `IS_` prefix from packets and any prefix
  from flags (such as `ISF_`) as we have a stand alone bitflags based struct to interact
  with.
- Where ever possible/reasonable we prefer rust native types:
  - Duration instead of milliseconds or seconds
  - bitflags over manual bit fiddling
- Where ever possible we provide higher level abstractions, such as (this is a
  non-exhaustive list):
  - [RaceLaps](crate::insim::RaceLaps) to avoid the user of the library needing to constantly reimplement protocol level rules.
  - [Connectionid](crate::identifiers::ConnectionId), [PlayerId](crate::identifiers::PlayerId) to avoid confusing plid and ucid.
  - [ObjectInfo](crate::insim::ObjectInfo) for layout objects
  - `Duration` instead of raw milliseconds/centiseconds/seconds
  - etc.

## Tokio vs blocking vs sans-io?

Selecting the right paradigm depends on your application's architecture, scalability requirements, and the specific features of the Live For Speed protocol you intend to use.

Where ever possible we've tried to keep compatibility between blocking and tokio
features almost identical. Infact the only change you need in 99.9% of cases is your
`read`, `write` and `connnect` calls need to been awaited.

### Tokio (Async)

The `tokio` feature is the default choice for modern Rust async networking.

It leverages an asynchronous runtime to manage non-blocking I/O, allowing your application to remain responsive while waiting for data to arrive from the simulator.

- Pick Tokio when: If you are building a high-performance server, such as a relay or a complex race management bot that must handle many concurrent connections or intensive background tasks without stalling.
- Trade-offs: It requires an async executor and introduces "function coloring," meaning your code must be async throughout the call chain to avoid blocking the runtime.

### Blocking (Synchronous)

The blocking feature provides a straightforward, procedural API where each network operation halts execution until it completes.

- Pick blocking when: Ideal for simple scripts, small command-line utilities, or "quick and dirty" race clients where procedural simplicity is prioritized over scalability. It is often sufficient for most race-day tools because "computers are fast actually," and a few OS threads are more than enough to handle race telemetry.
- Trade-offs: It has lower cognitive overhead than async code but the moment you want to do stuff in the background or you want to manage dozens of concurrent connections simultaneously it gets more complicated.

### Sans-IO (Bring your Own IO)

The "Sans-IO" approach is a design pattern that separates the protocol logic (state machine, packet parsing, encoding and connection upkeep) from the actual byte-shoveling of network sockets. In the insim crate, this core logic is exposed through [net::Codec](crate::net::Codec).

- Pick sans-io when:
  - Mixed Protocols: This is the only way to easily mix TCP (for reliable game state) and UDP (for high-frequency positional updates like MCI packets) within a single application loop.
  - Custom Runtimes: Pick this if you are using an alternative async runtime (like smol or async-std) or targeting restricted environments like WASM or no_std embedded hardware where standard network sockets are not available.
  - Testing: It is the best choice for complex testing, as you can verify logic deterministically without mocking sockets.
- Trade-offs: It requires you to implement your own "event loop" to drive the protocol forward, manually feeding bytes into the [Codec](crate::net::Codec) and pulling packets and connection upkeep information out.

## Establishing a connection

Establishing a connection is about as simple as it can be, if you use either of the 2
connection helpers:

- [tcp](crate::tcp)
- [udp](crate::udp)

These both return a [builder](crate::builder::Builder), which has a `connect` function (amongst others to handle setting values on the Isi / handshake packet) for both tokio and blocking
features, which will attempt to connect to Live For Speed.

This builder handles:

- The connection to LFS via tcp or udp.
- The initial handshake (creation and send of [Isi](crate::insim::Isi) packet).
- The upkeep of the connection (automatically responding to keep alive requests) - as
  long as the connection is polled (for tokio and blocking this means calling read
  reliably).

## Understanding the packet loop / Maintaining the connection

In the context of networking for Live For Speed (LFS), the "packet loop" is the heartbeat of your application. Since LFS uses a stateful connection (typically via InSim), the protocol expects a constant dialogue to confirm that your insim client is still "alive".

The most critical constraint of the LFS network stack is the timeout threshold. If the LFS server does not see activity from your connection-specifically, if you fail to acknowledge or poll the socket-it assumes a "ghost" connection and drops it.

We handle this for you, but only if you poll the read buffer.

> **Warning:** You must poll the read buffer at least once every 70 seconds.
> Failure to do so will cause Live For Speed to timeout the connection.
> We recommend at least every 30 seconds.

In a typical implementation, your loop would look something like this conceptually:

```rust,ignore
let conn = insim::tcp("127.0.0.1:29999").connect_async().await?;
loop {
    let packet = conn.read().await?;
    println!("{:?}", packet);
}
```

## Shortcuting the creation of packets

`insim` has a monolothic [Packet](crate::Packet) enum to help quickly and easily
identify all packets when receiving them, but also when sending.

Every Packet enum variant is conventionally a NewType.

However, it can be quite tedious to do this by hand.

Fortunately all packet variants payloads implement `Into<Packet>`.

Also fortunately many of the appropriate subtypes also implement `Into<Packet>`,
allowing you to just send those. For example, all the [TinyType](crate::insim::TinyType)
subtypes.

```rust,ignore
use insim::{Packet, insim::{Tiny, TinyType}};

// send will automatically take anything that converts into Packet, so these 3 are
// functionally the equivalent!
connection.send(Packet::Tiny(insim::insim::Tiny {
    subt: TinyType::None,
    ..Default::default()
}));

connection.send(insim::insim::Tiny {
    subt: TinyType::None,
    ..Default::default()
});

connection.send(TinyType::None);
```

It's also very common to want to send a [RequestId](crate::identifiers::RequestId)
against a packet. LFS will only send a reply to packets where `reqi` is non-zero, and
it echoes the value back so you can correlate a response to its original request.
To aid this process, you can import the [WithRequestId](crate::WithRequestId) trait:

```rust,ignore
use insim::{Packet, WithRequestId, identifiers::RequestId, insim::{Tiny, TinyType}};

// so these 3 are also functionally the equivalent!
connection.send(Packet::Tiny(Tiny {
    reqi: RequestId(1),
    subt: TinyType::None,
    ..Default::default()
}));

connection.send(Tiny {
    reqi: RequestId(1),
    subt: TinyType::None,
    ..Default::default()
});

connection.send(TinyType::None.with_request_id(1));
```

# Cookbook / Recipies

You can find a wide range of examples in the upstream repository under [examples/](https://github.com/theangryangel/insim.rs/tree/main/examples).

What you will find there:

- Connection basics (async and blocking TCP/UDP).
- Layout objects / AXM packet tooling.
- Live telemetry and in-game UI.
- Other protocols (Outsim and Outgauge).
- PTH tooling (PTH to SVG conversion).

If you're finding something missing, PRs are always welcome.

## Error handling

All connection read/write operations return `Result<_, insim::Error>`. Most errors are
fatal - the connection is no longer usable once one is returned and you should
reconnect. See [`crate::error::Error`] for the full list.

```rust,ignore
loop {
    match connection.read().await {
        Ok(packet) => { /* handle packet */ },
        Err(insim::Error::Disconnected) => {
            // LFS closed the connection (shutdown, kick, etc.) - reconnect
            break;
        },
        Err(insim::Error::Timeout(_)) => {
            // No data received within the timeout window - reconnect
            break;
        },
        Err(e) => return Err(e.into()),
    }
}
```

Errors from `write`/`send` calls are different: [`crate::error::Error::Encode`] means the
packet you built was invalid (e.g. too many objects in an [`crate::insim::Axm`]). The
connection is still alive in that case.

## Spawning a background task

For Tokio applications, [`crate::builder::Builder::spawn`] runs the connection in a
background task and gives you a cloneable [`crate::builder::InsimTask`] handle with
`send` / `subscribe` / `shutdown` methods. This is the most convenient pattern for
applications that need to send packets from many places:

```rust,ignore
let (task, join_handle) = insim::tcp("127.0.0.1:29999")
    .isi_flag_mci(true)
    .isi_interval(Duration::from_secs(1))
    .spawn(100) // channel capacity
    .await?;

let mut events = task.subscribe();

tokio::spawn(async move {
    while let Ok(packet) = events.recv().await {
        if let insim::Packet::Mci(mci) = packet {
            for car in &mci.info {
                println!("plid={} speed={:.1}m/s", car.plid, car.speed.to_meters_per_sec());
            }
        }
    }
});

// send from anywhere you have a clone of `task`
task.send_message("Hello!", None).await?;

// graceful shutdown
task.shutdown().await;
join_handle.await??;
```

## Receiving telemetry (MCI)

MCI requires the [`crate::insim::IsiFlags::MCI`] flag and a non-zero interval set
during the handshake. Each [`crate::insim::Mci`] packet carries a `Vec` of
[`crate::insim::CompCar`] entries, one per car currently on track:

```rust,ignore
let mut connection = insim::tcp("127.0.0.1:29999")
    .isi_flag_mci(true)
    .isi_interval(Duration::from_millis(200))
    .connect_async()
    .await?;

loop {
    let packet = connection.read().await?;

    if let insim::Packet::Mci(mci) = packet {
        for car in &mci.info {
            println!(
                "plid={} speed={:.1}m/s x={} y={} z={}",
                car.plid,
                car.speed.to_meters_per_sec(),
                car.x,
                car.y,
                car.z,
            );
        }
    }
}
```

## Tracking player join / leave

Connections ([`crate::insim::Ncn`]) and players ([`crate::insim::Npl`]) are separate
events. A connection must exist before a player can, but a connection can outlast its
player (e.g. after pitting to spectate). To get an initial snapshot of all current
connections and players, send [`crate::insim::TinyType::Ncn`] and
[`crate::insim::TinyType::Npl`] after connecting:

```rust,ignore
use insim::{WithRequestId, insim::TinyType};

// request current state - replies arrive as normal Ncn/Npl packets with reqi echoed
connection.send(TinyType::Ncn.with_request_id(1)).await?;
connection.send(TinyType::Npl.with_request_id(1)).await?;

loop {
    match connection.read().await? {
        insim::Packet::Ncn(ncn) => println!("connection joined: ucid={}", ncn.ucid),
        insim::Packet::Cnl(cnl) => println!("connection left:   ucid={}", cnl.ucid),
        insim::Packet::Npl(npl) => println!("player joined:     plid={} ucid={}", npl.plid, npl.ucid),
        insim::Packet::Pll(pll) => println!("player left:       plid={}", pll.plid),
        _ => {},
    }
}
```

## Sending messages

[`crate::builder::InsimTask::send_message`] picks the right packet type automatically.
If you need direct control:

```rust,ignore
use insim::insim::{Mst, Mtc};
use insim::identifiers::ConnectionId;

// broadcast to all (up to 64 chars via Msx, longer via Mst)
connection.send(Mst { msg: "Hello track!".into(), ..Default::default() }).await?;

// private message to one connection
connection.send(Mtc {
    ucid: ConnectionId(3),
    text: "Hello driver!".into(),
    ..Default::default()
}).await?;
```

## Colouring strings

`insim` re-exports string helpers from the internal core crate for convenience, allowing
you to quickly build strings with colours, without having to constantly lookup what
colours are what numbers..

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

The insim crate now provides higher level layout object abstraction, allowing you to
just get busy building stuff.

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

```rust,ignore
use insim::insim::{Axm, PmoAction, PmoFlags};
use insim::core::object::{ObjectInfo, ObjectCoordinate};
use insim::core::object::tyres::{Tyres, TyreColour};
use insim::core::heading::Heading;

connection.write(Axm {
    action: PmoAction::AddObjects(vec![
        ObjectInfo::TyreStack4(Tyres {
            xyz: ObjectCoordinate::new(0, 0, 0),
            colour: TyreColour::Blue,
            floating: false,
            heading: Heading::ZERO,
        })
    ]),
    ..Default::default()
}).await?;
```
