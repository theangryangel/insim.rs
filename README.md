# insim.rs

A collection of crates to assist working with the [Live For Speed](https://lfs.net/) racing simulator and it's [Insim](https://en.lfsmanual.net/wiki/InSim.txt) (protocol).

For the foreseeable future only TCP is supported for simplicity and time.
Outsim is currently not supported.

:warning: The API is not yet 100% stable at this time. Use at your own risk.

| Crate                                  | Usage                                                                                                                                                                                                  |
| -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `insim`                                | Transport and protocol implementation. If you are looking to implement something this is likely the crate you want.                                                                                    |
| `insim_core`                           | Contains core types shared across other crates.                                                                                                                                                        |
| `insim_derive`                         | Contains proc_macros for insim_core.                                                                                                                                                                   |
| `insim_game_data`                      | Contains track information.                                                                                                                                                                            |
| `insim_pth`                            | Implements a PTH file read/writer.                                                                                                                                                                     |
| `insim_smx`                            | Implements a SMX file reader/writer.                                                                                                                                                                   |
| `multi_index` and `multi_index_derive` | `multi_index` started out life as a fork of [lun3x/multi_index_map](https://github.com/lun3x/multi_index_map) with the intention to resubmit upstream. See multi_index/README.md for more information. |

## Documentation

Until we're released please run `cd insim && cargo doc --no-deps --open`

## TODO

- `git grep '\(TODO\|FIXME\|XXX\)'`
- check out the issues list
