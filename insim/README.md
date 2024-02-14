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
detailed reference for what each packet describes and can do. Where possible
this crate aligns the naming of fields in packets to match the original spec.
In a handful of circumstances we have needed to rename, or separate some fields
(most notably thrbrk and cluhan in the [Con](crate::Packet::Con) packet).

## Supported Features

- insim over TCP or UDP
- insim over TCP and Websocket via LFS World Relay

## Feature flags

The following are a list of [Cargo features][cargo-features] that can be enabled or disabled:

| Name      | Description                                                                 | Default? |
| --------- | --------------------------------------------------------------------------- | -------- |
| serde     | Enable serde support                                                        | No       |
| pth       | Pull in insim_pth and re-export                                             | No       |
| smx       | Pull in insim_smx and re-export                                             | No       |
| tokio     | Enable tokio support                                                        | Yes      |
| blocking  | Enable blocking/sync support                                                | Yes      |
| websocket | Enable LFSW Relay support over websocket using Tungstenite (requires tokio) | Yes      |

## Making a TCP connection (using tokio)

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

## Making a TCP connection (using blocking)

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

## Making a LFS World Relay connection (using tokio)

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

## Making a UDP connection (using tokio)

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

## Additional examples

For further examples see <https://github.com/theangryangel/insim.rs/tree/main/examples>
