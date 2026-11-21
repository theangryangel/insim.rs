# simple-blocking

Demonstrates creating standard-library TCP or UDP sockets, configuring an `Isi`
packet, and starting a blocking InSim `Session` with an explicit handshake.
MCI updates default to once per second.

## Usage

- Check `cargo run -- --help`

TCP only:

```sh
cargo run -p simple-blocking -- tcp --addr 127.0.0.1:29999
```

TCP control with MCI/NLP received over an attached UDP socket:

```sh
cargo run -p simple-blocking -- tcp --addr 127.0.0.1:29999 --udp-bind 0.0.0.0:0
```

Port `0` selects an available local port, which is passed to LFS in `Isi.udpport`.
Choose a fixed port instead when needed, for example `--udp-bind 0.0.0.0:30000`.
Packets from both sockets arrive through the same `session.read()` loop.
