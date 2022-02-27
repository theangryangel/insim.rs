# insim.rs

[Live For Speed](https://lfs.net/) racing simulator [Insim](https://en.lfsmanual.net/wiki/InSim.txt) (protocol) codec, lower level transport, higher level client and associated utilities implemented in pure Rust.

For the foreseeable future only TCP is supported for simplicity and time.

:warning: The high level API is not yet 100% stable at this time. Use at your own risk.

## Documentation

Until we're released please run `cargo doc --no-deps --open`

## Examples

- High level "framework" api: `cargo run --example relay`
- Low level "transport" api: `cargo run --example transport`

## TODO

- `git grep '\(TODO\|FIXME\)'`
