# insim.rs

A collection of crates to assist working with the [Live For Speed](https://lfs.net/)
racing simulator and it's [Insim](https://en.lfsmanual.net/wiki/InSim.txt) (protocol).

The intention is to provide a strongly typed, native rust implementation, rather
than a thin layer over a series of bytes and primitive types.

If you're not sure where to start, you probably want to look at the [examples](https://github.com/theangryangel/insim.rs/tree/main/examples).

| Crate          | Usage                                                  |
| -------------- | ------------------------------------------------------ |
| `insim`        | Insim connection and protocol implementation.          |
| `insim_core`   | Contains core types shared across other crates.        |
| `insim_macros` | Contains proc_macros republished through insim_core.   |
| `insim_pth`    | Implements a PTH file read/writer.                     |
| `insim_smx`    | Implements a SMX file reader/writer.                   |
| `outgauge`     | Implements "sans-io" Outgauge protocol implementation. |

## Thanks

- [simbroadcasts/node-insim](https://github.com/simbroadcasts/node-insim) which I used
  to bootstrap many of the packet unit tests.
- [LFS](https://www.lfs.net/) and it's community, without which this project would not exist.
