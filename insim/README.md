# insim

insim is a Rust library for working with the Racing Simulator Live For Speed.

It's primary use case is to communicate with LFS via Insim, however it also provides
additional utilities for working with LFS as a whole through feature flags and
it's sibling crates.

The intention is to provide a strongly typed, native rust implementation, rather
than a thin layer over a series of bytes.

## Supported Features

- insim over TCP or UDP
- insim over TCP and Websocket via LFS World Relay

## Feature flags

- `serde`: Enable serde support
- `pth`: Pull in insim_pth and re-export
- `smx`: Pull in insim_smx and re-export
- `websocket`: Enable LFSW Relay support over websocket (default)

## Examples

See examples directory.

## Breaking Changes

- theangryangel/insim.rs#127 restructures the crate to more closely
  align with the std library:
  - `insim::network` was renamed to `insim::net`
  - `insim::codec::Codec` was moved to `insim::net::Codec`
  - `insim::codec::Mode` was moved to `insim::net::Mode`
  - Several convenience aliases were added: `insim::{Result, Error, Packet}`
  - :warning: `insim::connection` was removed in favour of the shortcut methods:
    `insim::tcp`, `insim::udp`, `insim::relay` which now returns a reusable builder.
    It does not auto-reconnect.
  - `Network` trait was renamed to `TryReadWriteBytes` to better indicate its
    usage. Network was too generic.
- :warning: It is now your responsibility to escape and unescape strings
  For convenience insim re-exports [insim::core::string::escape] and
  [insim::core::string::unescape].
  See theangryangel/insim.rs#92 for further info.
