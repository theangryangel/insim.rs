# pyinsim

Python bindings for [insim.rs](https://github.com/theangryangel/insim.rs), built with [PyO3](https://pyo3.rs) and [maturin](https://maturin.rs).

## Development setup

```bash
cd pyinsim
uv sync --dev
maturin develop
```

## Packet type codegen

`pyinsim/insim_schema.json` and `python/pyinsim/_types.py` are auto-generated
from the Rust packet types and **checked into git**. Regenerate both after any
change to packet structs in the `insim` crate:

```bash
cargo run -p xtask-pyinsim-schema
```

Then commit both changed files. CI verifies they are current with:

```bash
cargo run -p xtask-pyinsim-schema -- --check
```

## Usage

### Connecting and handling packets

Use `InsimClient` as an async context manager — it disconnects automatically on exit.
Register typed handlers with `@client.on`:

```python
import asyncio
from insim_rs import InsimClient
from insim_rs._types import Ncn, Mso

async def main() -> None:
    async with InsimClient("127.0.0.1:29999") as client:

        @client.on(Ncn)
        async def on_join(packet: Ncn) -> None:
            print(f"[join] {packet.pname} ({packet.uname})")

        @client.on(Mso)
        async def on_chat(packet: Mso) -> None:
            print(f"[chat] {packet.msg}")

        await client.run()

asyncio.run(main())
```

### Reusable handlers

Group related handlers into a `Handler` and attach it to one or more clients:

```python
from insim_rs import InsimClient
from insim_rs.handler import Handler
from insim_rs._types import Ncn

joins = Handler()

@joins.on(Ncn)
async def on_join(packet: Ncn) -> None:
    print(f"[join] {packet.pname}")

async def main() -> None:
    async with InsimClient("127.0.0.1:29999") as client:
        client.include_handler(joins)
        await client.run()
```

### Streaming a packet type

`client.stream(PacketType)` is an async generator that yields every matching
packet. The subscription is cleaned up automatically when the loop exits:

```python
async def main() -> None:
    async with InsimClient("127.0.0.1:29999") as client:
        async for packet in client.stream(Mso):
            print(f"[chat] {packet.msg}")
            if packet.msg == "!quit":
                break
```

### Middleware

Middleware runs before handlers and receives every packet regardless of type —
useful for logging or metrics:

```python
from insim_rs import InsimClient, AnyPacket

async def main() -> None:
    async with InsimClient("127.0.0.1:29999") as client:

        @client.middleware
        async def log_all(packet_type: str, packet: AnyPacket) -> None:
            print(f"← {packet_type}")

        await client.run()
```

## Running tests

```bash
pytest pyinsim/tests/
```
