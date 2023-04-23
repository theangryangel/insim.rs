# insim.rs

A collection of crates to assist working with the [Live For Speed](https://lfs.net/) racing simulator and it's [Insim](https://en.lfsmanual.net/wiki/InSim.txt) (protocol).

Support is currently available for:

- Insim over TCP
- Insim over UDP
- Insim over TCP via LFS World Relay

The sibling protocols, Outsim and Outgauge, are currently not supported. LFS
World websocket support is being considered.

:warning: The API is not yet stable at this time. Use at your own risk. I am
currently not touching the version number.

| Crate                                  | Usage                                                                                                                                                                                                  |
| -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `insim`                                | Connection and protocol implementation.                                                                                                                                                                |
| `insim_extras`                         | Utilities to make using insim easier.                                                                                                                                                                  |
| `insim_core`                           | Contains core types shared across other crates.                                                                                                                                                        |
| `insim_derive`                         | Contains proc_macros for insim_core.                                                                                                                                                                   |
| `insim_game_data`                      | Contains track information.                                                                                                                                                                            |
| `insim_pth`                            | Implements a PTH file read/writer.                                                                                                                                                                     |
| `insim_smx`                            | Implements a SMX file reader/writer.                                                                                                                                                                   |
| `multi_index` and `multi_index_derive` | `multi_index` started out life as a fork of [lun3x/multi_index_map](https://github.com/lun3x/multi_index_map) with the intention to resubmit upstream. See multi_index/README.md for more information. |

## Documentation

Until we're released, either:

- Please run `cd insim && cargo doc --no-deps --open`
- Or take a look at `insim/examples/transport`

## TODO

- `git grep '\(TODO\|FIXME\|XXX\)'`
- check out the issues list
