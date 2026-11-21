A friendly, Rust idiomatic library for the Outsim protocol used by [Live for Speed](https://www.lfs.net/) racing simulator.

The focus of this library is providing a high level, strongly typed, primitives that are difficult to misuse and have reasonable performance, rather than be a thin layer over a series of bytes.

Where possible this crate aligns the naming of fields in packets to match the [original Outsim specification](https://en.lfsmanual.net/wiki/InSim.txt).

# High-level features

- "sans-io" implementation of Outsim

# Related crates

You might also find these related crates useful:

- `insim` - for interacting with LFS over Insim
- `insim_pth` – for reading and writing LFS PTH files
- `insim_smx` – for reading and writing LFS SMX files
- `outgauge` - "sans-io" implementation of the LFS outgauge protocol

They follow the same design focus and can be found in the same GitHub repository.

# Examples

Examples can be found in the GitHub repository.

# Selecting the OutSim format

`OutSimCodec::default()` decodes original OutSim packets, with or without an ID,
returning `OutsimPack` directly. `OutSimCodec::extended(opts)` decodes only the
extended layout selected by the sender's OutSim Opts, returning `OutsimPack2`.
There is no format auto-detection or intermediate packet enum.

Standalone receivers make the same explicit choice:

```rust,ignore
let mut original = outsim::Listener::udp(socket)?;
let packet: outsim::OutsimPack = original.read().await?;

let mut extended = outsim::Listener::extended(other_socket, opts)?;
let packet: outsim::outsimpack2::OutsimPack2 = extended.read().await?;
```

Standard-library sockets use blocking `read()` calls. Listeners have no inactivity
timeout and can be consumed by `Session::attach`. A Demux converts their selected
packet type into the session's corresponding telemetry variant.

The extended options must match LFS's configuration. With `OutSimOpts::HEADER`,
`OSIM` identifies extended datagrams; headerless layouts are recognized by their
configured length. Formats without unique markers may overlap with other UDP
protocols, so decoder ordering or separate sockets still matters.
