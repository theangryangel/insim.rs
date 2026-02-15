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
  - Quickly create connections using `tcp` or `udp` functions.
  - If you prefer to handle that yourself, you can just use `net::Codec`.
  - Mixing and matching TCP and UDP for positional updates is possible, but requires you to drop to the "sans-io" approach.
- Handles the connection upkeep (keepalives / ping packets, etc.) automatically for you.
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

```rust,ignore
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
- `kitcar` - a Work in Progress, unstable suite of micro libraries to help build
  multi-player mini-games quickly using insim.

They follow the same design focus and can be found in the same GitHub repository, or on
crates.io.
