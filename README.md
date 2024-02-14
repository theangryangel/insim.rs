# insim.rs

A collection of crates to assist working with the [Live For Speed](https://lfs.net/)
racing simulator and it's [Insim](https://en.lfsmanual.net/wiki/InSim.txt) (protocol).

The intention is to provide a strongly typed, native rust implementation, rather
than a thin layer over a series of bytes and primitive types.

If you're not sure where to start, you probably want to look at the [examples](https://github.com/theangryangel/insim.rs/tree/main/examples).

| Crate        | Usage                                           | Documentation              |
| ------------ | ----------------------------------------------- | -------------------------- |
| `insim`      | Connection and protocol implementation.         | https://docs.rs/insim      |
| `insim_core` | Contains core types shared across other crates. | https://docs.rs/insim_core |
| `insim_pth`  | Implements a PTH file read/writer.              | https://docs.rs/insim_pth  |
| `insim_smx`  | Implements a SMX file reader/writer.            | https://docs.rs/insim_smx  |

If you're looking for race_directord that has been moved to <https://github.com/theangryangel/race_directord/>.

## TODO

- `git grep '\(TODO\|FIXME\|XXX\)'`
- check out the issues list
