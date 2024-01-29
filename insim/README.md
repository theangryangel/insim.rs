# insim

insim is a Rust library for working with the Racing Simulator Live For Speed.

It's primary use case is to communicate with LFS via Insim, however it also provides
additional utilities for working with LFS as a whole through feature flags and
it's sibling crates.

The intention is to provide a strongly typed, native rust implementation, rather
than a thin layer over a series of bytes.

Many of the core types, such as [Vehicle](crate::core::vehicle::Vehicle), [Track](crate::core::track::Track), etc. have been housed within
the crate `insim_core`, which is re-exported.

You will probably want to use <https://en.lfsmanual.net/wiki/InSim.txt> as a
detailed reference for what each packet describes and can do.

## Supported Features

- insim over TCP or UDP
- insim over TCP and Websocket via LFS World Relay

## Feature flags

The following are a list of [Cargo features][cargo-features] that can be enabled or disabled:

- `serde`: Enable serde support
- `pth`: Pull in insim_pth and re-export
- `smx`: Pull in insim_smx and re-export
- `websocket`: Enable LFSW Relay support over websocket (default) using
  Tungstenite

## Making a TCP connection

```rust
let conn = insim::tcp("127.0.0.1:29999").connect().await?;
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

## Making a UDP connection

```rust
let conn = insim::udp("127.0.0.1:29999", None).connect().await?;
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

## Making a LFS World Relay connection

```rust
let conn = insim::relay()
    .relay_select_host("Nubbins AU Demo")
    .relay_websocket(true)
    .connect()
    .await?;

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

## Additional examples

For further examples see <https://github.com/theangryangel/insim.rs/tree/main/insim/examples>

## Breaking changes

- [#140](https://github.com/theangryangel/insim.rs/issues/140) renamed a significant proportion of
  the `insim::Packet` enum, and silbing structs and enums to more closely align with the upstream
  spec (`Insim.txt`).
- [#127](https://github.com/theangryangel/insim.rs/issues/127) restructures the crate to more closely
  align with the std library:
  - `insim::network` was renamed to `insim::net`
  - `insim::codec::Codec` was moved to `insim::net::Codec`
  - `insim::codec::Mode` was moved to `insim::net::Mode`
  - Several convenience aliases were added: `insim::{Result, Error, Packet}`
  - `Network` trait was renamed to `TryReadWriteBytes` to better indicate its
    usage. Network was too generic.
- [#127](https://github.com/theangryangel/insim.rs/issues/127) `insim::connection` was removed in favour
  of the shortcut methods: `insim::tcp`, `insim::udp`, `insim::relay` which now return a reusable builder.
  These do not auto-reconnect.
- [#92](https://github.com/theangryangel/insim.rs/issues/92) disables the automatic escaping and
  unescaping of strings. For convenience the insim crate now re-exports
  `insim::core::string::escape` and `insim::core::string::unescape`.
