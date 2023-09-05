# insim

insim is a Rust library for working with the Racing Simulator Live For Speed.

It's primary use case is to communicate with LFS via Insim, however it also provides
additional utilities for working with LFS as a whole through feature flags and
it's sibling crates.

## Supported Features

- insim over TCP or UDP
- insim over TCP and Websocket via LFS World Relay

## Feature flags

- `serde`: Enable serde support
- `game_data`: Pull in insim_game_data and re-export
- `pth`: Pull in insim_pth and re-export
- `smx`: Pull in insim_smx and re-export

## Examples

See examples directory.

## Breaking Changes

- :warning: It is now your responsibility to escape and unescape strings
  For convenience insim re-exports [insim::tools::string::escape] and
  [insim::tools::string::unescape].
  See theangryangel/insim.rs#92 for further info.
