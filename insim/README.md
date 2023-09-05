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
