# insim.rs

A collection of crates to assist working with the [Live For Speed](https://lfs.net/) racing simulator and it's [Insim](https://en.lfsmanual.net/wiki/InSim.txt) (protocol).

:warning: The API is not yet stable at this time. Use at your own risk. I am
currently not touching the version number.

If you're not sure where to start, you probably want to look at the README and examples within the `insim` crate.

| Crate          | Usage                                           |
| -------------- | ----------------------------------------------- |
| `insim`        | Connection and protocol implementation.         |
| `insim_core`   | Contains core types shared across other crates. |
| `insim_derive` | Contains proc_macros for insim_core.            |
| `insim_pth`    | Implements a PTH file read/writer.              |
| `insim_smx`    | Implements a SMX file reader/writer.            |

## Documentation

Until we're released, either:

- Please run `cargo doc --no-deps --open`
- Take a look at the examples in each crate

## TODO

- `git grep '\(TODO\|FIXME\|XXX\)'`
- check out the issues list
