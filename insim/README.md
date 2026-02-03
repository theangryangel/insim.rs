A friendly, Rust idiomatic library for the InSim protocol used by [Live for Speed](https://www.lfs.net/) racing simulator.

The focus of this library is providing a high level, strongly typed primitives that are difficult to misuse and have reasonable performance, rather than be a thin layer over a series of bytes.

We prioritise compatibility with the latest LFS protocol specifications. Given LFS is now effectively in an evergreen release cycle, support for superseded protocol versions is systematically removed in new releases, which is a key measure for reducing technical debt and maintenance burden.

Where possible this crate aligns the naming of fields in packets to match the [original Insim specification](https://en.lfsmanual.net/wiki/InSim.txt).

In a handful of circumstances we have needed to rename, or separate some fields to align with the crate's key focus.

# Key API Surface

- `Packet`, `WithRequestId` - strongly typed packet handling.
- `tcp`, `udp`, `net::Codec` - connection builders and sans-io codec.
- `Colour`, `Escape` - string helpers for colour and escaping.

# High-level features

Here is a non-exhaustive list of the things that `insim` supports:

- insim over TCP or UDP (for both blocking and tokio). Mixing and matching TCP and UDP
  for positional updates is possible, but requires you to drop to the "sans-io"
  approach.
- LFSW Relay support was removed due to the upstream service ceasing.
- Or sans-io/bring-your-own-IO of your own choice through the [crate::net::Codec].

# Usage

`insim` is on crates.io and can be used by adding `insim` to your dependencies in your project’s Cargo.toml.
Or more simply, just run `cargo add insim`.

If you want to use an unreleased version you can also reference the GitHub repository.

# Related crates

You might also find these related crates useful:

- `insim_pth` – for reading and writing LFS PTH files
- `outgauge` - "sans-io" implementation of the LFS outgauge protocol
- `outsim` - "sans-io" implementation of the LFS outsim protocol

They follow the same design focus and can be found in the same GitHub repository.

# Quick Start

You can find a wide range of examples in the upstream repository under `examples/`:
<https://github.com/theangryangel/insim.rs/tree/main/examples>

## Tokio (Async)

### TCP Connection

```rust
let conn = insim::tcp("127.0.0.1:29999").connect_async().await?;
loop {
    let packet = conn.read().await?;
    println!("{:?}", packet);

    match packet {
        insim::Packet::Mci(_) => {
          println!("Got a MCI packet!")
        },
        _ => {},
    }
}
```

### UDP Connection

```rust
let conn = insim::tcp("127.0.0.1:29999", None).connect_async().await?;
loop {
    let packet = conn.read().await?;
    println!("{:?}", packet);

    match packet {
        insim::Packet::Mci(_) => {
          println!("Got a MCI packet!")
        },
        _ => {},
    }
}
```

## Blocking

### TCP Connection

```rust
let conn = insim::tcp("127.0.0.1:29999").connect()?;
loop {
    let packet = conn.read()?;
    println!("{:?}", packet);

    match packet {
        insim::Packet::Mci(_) => {
          println!("Got a MCI packet!")
        },
        _ => {},
    }
}
```

## String helpers

`insim` re-exports string helpers from the internal core crate for convenience, allowing
you to quickly build formatted and escaped strings.

```rust
use insim::{Colour, Escape};

let message = "Hello".red();
let escaped = "^|*".escape();
let unescaped = escaped.unescape();
```

## Concepts

- Connections can be TCP/UDP or handled manually via the sans-io `net::Codec`.
- Packets are represented by the `Packet` enum, with helpers like `WithRequestId`.
- Many of the types within a `Packet` variant implement `Into<Packet>`, allowing you to
  avoid complex and tedious variable construction.
- String utilities cover colours and escaping, plus codepage handling in the core crate.

# Examples

You can find a wide range of examples in the upstream repository under `examples/`:
<https://github.com/theangryangel/insim.rs/tree/main/examples>

What you will find there:

- Connection basics (async and blocking TCP/UDP).
- Layout objects / AXM packet tooling.
- Live telemetry and in-game UI.
- Other protocols (Outsim and Outgauge).
- PTH tooling (PTH to SVG conversion).

For sans-io usage, see [`net::Codec`](crate::net::Codec).

# Crate features

| Name       | Description                  | Default? |
| ---------- | ---------------------------- | -------- |
| `serde`    | Enable serde support         | No       |
| `tokio`    | Enable tokio support         | Yes      |
| `blocking` | Enable blocking/sync support | Yes      |

# Internal core crate

`insim_core` contains shared low-level types and utilities. Most users should only depend on `insim`.
