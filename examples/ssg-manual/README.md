# ssg-manual

Demonstrates TCP control with MCI/NLP, OutGauge and original OutSim sharing one
attached UDP socket. A `Demux` selects the decoder, and all packets arrive through
the same `Session::read()` loop. There is no manual datagram parsing or separate
UDP receive loop.

After the handshake, `SMALL_SSG` and `SMALL_SSP` request OutGauge and original OutSim
updates once per second. MCI is enabled through the handshake options.

## Usage

- Check `cargo run -- --help`

```sh
cargo run -p ssg-manual -- --addr 127.0.0.1:29999 --udp-listen-addr 0.0.0.0:0
```

Port `0` selects an available local port; its actual value is passed to LFS in
`Isi.udpport`. A fixed port can also be supplied.

SSG and SSP send OutGauge and OutSim to `Isi.udpport`, alongside MCI and NLP.
Telemetry is recognized by size, so the stack tries OutGauge, original OutSim,
then InSim. Known issue: an NLP for 10 or 15 players has the same size as OutSim
or OutGauge and is decoded as telemetry. The example does not enable NLP.

For extended OutSim, configure `OutSimCodec::extended(opts)` with the same OutSim Opts
as the sender. That decoder returns `OutsimPack2` directly; the default decoder returns
`OutsimPack`.
