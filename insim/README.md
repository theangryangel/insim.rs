A friendly, Rust idiomatic library for the InSim protocol used by [Live for Speed](https://www.lfs.net/) racing simulator.

The focus of this library is providing a high level, strongly typed, primitives that are difficult to misuse and have reasonable performance, rather than be a thin layer over a series of bytes.

Where possible this crate aligns the naming of fields in packets to match the [original Insim specification](https://en.lfsmanual.net/wiki/InSim.txt).

In a handful of circumstances we have needed to rename, or separate some fields to align with the crate's key focus.

# High-level features

Here is a non-exhaustive list of the things that `insim` supports:

- insim over TCP or UDP
- insim over TCP and Websocket via LFS World Relay
- Optional Blocking or Tokio based async functionality
- PTH file parsing
- SMX file parsing

# Usage

`insim` is on crates.io and can be used by adding `insim` to your dependencies in your project’s Cargo.toml.
Or more simply, just run `cargo add insim`.

If you want to use an unreleased version you can also reference the GitHub repository.

# Related crates

You might also find these related crates useful:

- `insim_pth` – for reading and writing LFS PTH files
- `insim_smx` – for reading and writing LFS SMX files
- `outgauge` - "sans-io" implementation of the LFS outgauge protocol
- `outsim` - "sans-io" implementation of the LFS outsim protocol

They follow the same design focus and can be found in the same GitHub repository.

# Examples

Looking for more examples? Take a look at our more detailed examples here: <https://github.com/theangryangel/insim.rs/tree/main/examples> – it's full of practical code to help you get started.

## Async TCP Connection

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

## Blocking TCP Connection

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

## Async LFSW Relay Connection

```rust
let conn = insim::relay()
    .relay_select_host("Nubbins AU Demo")
    .connect_async()
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

## Async UDP Connection

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

# Crate features

The following are a list of [Cargo features][cargo-features] that can be enabled or disabled:

| Name        | Description                                                                           | Default? |
| ----------- | ------------------------------------------------------------------------------------- | -------- |
| `serde`     | Enable serde support                                                                  | No       |
| `tokio`     | Enable tokio support                                                                  | Yes      |
| `blocking`  | Enable blocking/sync support                                                          | Yes      |
| `relay`     | Enable LFSW Relay support                                                             | Yes      |
| `websocket` | Enable LFSW Relay support over websocket using Tungstenite (requires tokio and relay) | No       |
