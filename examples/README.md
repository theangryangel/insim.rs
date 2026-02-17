# Examples

This folder contains various examples showing how to use the insim crates.
Each example is setup as its own crate so its dependencies are clear.

## Common prerequisites

- Start LFS before running InSim examples.
- Use `/insim 29999` in chat to enable InSim unless the example says otherwise.
- Run each example with `cargo run -- --help` to see its flags.

## Catalog

- `simple-async` - Async TCP connection and basic packet loop.
- `simple-blocking` - Blocking TCP connection and basic packet loop.
- `strobe` - Toggle local vehicle lights in a sequence.
- `marquee` - Layout objects and AXM packet tooling experiments.
- `live-delta` - Live delta timing displayed with in-game buttons.
- `btn-clickid-mangled-mvp` - Minimal repro: update same ClickID from clickable to non-clickable.
- `clockwork-carnage` - Kitcar proof-of-concept (WIP).
- `simple-outsim` - Basic outsim usage.
- `simple-outgauge` - Basic outgauge usage.
- `pth2svg` - Convert a LFS PTH file to an SVG.
- `ssg-manual` - Handle split TCP and UDP connection to LFS, manually, allowing you to receive
  Mci, Nlp, Outsim and Outgauge packets via UDP, and everything else via TCP.
