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
