# insim_o3

Python bindings for [insim.rs](https://github.com/theangryangel/insim.rs), built with [PyO3](https://pyo3.rs) and [maturin](https://maturin.rs).

## Development setup

Install dependencies and build the native extension in-place:

```bash
cd insim_o3
uv sync --dev
uv run maturin develop
```

`maturin develop` compiles the Rust extension and installs it into the virtual
environment created by `uv sync`. Re-run it any time you change Rust source
files under `src/`.

## Packet type codegen

`insim_o3/insim_schema.json` and `python/insim_o3/packets.py` are auto-generated
from the Rust packet types and **checked into git**. Regenerate both after any
change to packet structs in the `insim` crate:

```bash
cargo run -p xtask-insim-o3-schema
```

Then commit both changed files. CI verifies they are current with:

```bash
cargo run -p xtask-insim-o3-schema -- --check
```

## Usage

The library is async at the core. For standalone bots there's a sync entry
point (`run_forever`); for embedding in another asyncio program, use `Insim`
as an async context manager.

### Standalone bot

```python
from insim_o3 import Insim
from insim_o3.packets import Ncn, Mso

client = Insim("127.0.0.1:29999")

@client.on(Ncn)
async def on_join(packet: Ncn) -> None:
    print(f"[join] {packet.pname} ({packet.uname})")

@client.on(Mso)
async def on_chat(packet: Mso) -> None:
    print(f"[chat] {packet.msg}")

client.run_forever()
```

`run_forever()` connects, dispatches until the connection drops or Ctrl+C,
and tears down cleanly. Handler functions may be sync or `async def`.

### Embedding in another asyncio program

When you need to run alongside other async tasks (a web server, another
client, etc.), use `Insim` as a context manager directly:

```python
import asyncio
from insim_o3 import Insim

async def main() -> None:
    async with Insim("127.0.0.1:29999") as client:
        @client.on(Ncn)
        async def on_join(packet: Ncn) -> None: ...

        await client.run()

asyncio.run(main())
```

### Connection options

```python
from insim_o3 import Insim
from insim_o3.packets import IsiFlag

client = Insim(
    "127.0.0.1:29999",
    flags=[IsiFlag.MCI, IsiFlag.MSO_COLS],
    iname="my_app",
    admin_password="secret",
    interval_ms=500,
    prefix="!",      # route !-prefixed chat as Mso with usertype=Prefix
    capacity=256,    # broadcast channel buffer size
)
```

### Sending packets

```python
from insim_o3.packets import Mtc, SoundType

await client.send(Mtc(ucid=packet.ucid, plid=packet.plid, sound=SoundType.SysMessage, text="hello"))
```

### Reusable handlers

Group related handlers into a `handler.Handler` subclass and decorate
methods with `@handler.on(PacketType)`. Method names are arbitrary, and one
method may register for multiple packet types (`@handler.on(Ncn, Cnl)`):

```python
from insim_o3 import Insim, handler
from insim_o3.packets import Ncn, Mso

class Bot(handler.Handler):
    @handler.on(Ncn)
    async def announce(self, packet: Ncn) -> None:
        print(f"[join] {packet.pname}")

    @handler.on(Mso)
    async def chat(self, packet: Mso) -> None:
        print(f"[chat] {packet.msg}")

async with Insim("127.0.0.1:29999") as client:
    client.handlers.add(Bot())
    await client.run()
```

`client.handlers` is a small registry: `add(handler)` and `remove(handler)`
work directly, and the registry is iterable.

The same `on` verb is used as a method on the client itself
(`@client.on(PacketType)` shown earlier in the standalone-bot example) - at
the client level it registers a callback at runtime against the default
handler; on a `Handler` subclass it registers a method at class-definition
time.

### Middleware

Middleware runs before handlers and receives every packet regardless of type -
useful for logging or metrics. Sync and `async def` middleware are both
supported:

```python
from insim_o3 import Insim
from insim_o3.dispatcher import AnyPacket

async with Insim("127.0.0.1:29999") as client:

    @client.middleware.add
    async def log_all(packet: AnyPacket) -> None:
        print(f"<- {type(packet).__name__}")

    await client.run()
```

`client.middleware` is a registry just like `client.handlers`:
`@client.middleware.add` is the decorator form; `client.middleware.remove(fn)`
deregisters.

### Error handling

Exceptions raised by individual handlers or middleware are caught, logged,
and skipped so one bad callback does not kill the dispatch loop. Override
the policy with the `on_error` kwarg on `Insim` (and `Handler`):

```python
def reraise(exc: BaseException, packet: object, fn: object) -> None:
    raise exc

async with Insim("127.0.0.1:29999", on_error=reraise) as client:
    ...
```

The default writes via the standard `logging` module under the
`insim_o3.handler` logger.

### Logging

The Rust core emits `tracing` events that are forwarded into Python's
`logging` module via `pyo3-log`. Configure them like any other Python
logger:

```python
import logging

logging.basicConfig(level=logging.INFO)

# Quiet the noisy packet-decode log if you don't need it:
logging.getLogger("insim.net.codec").setLevel(logging.WARNING)
```

Logger names mirror the Rust module path (e.g. `insim.net.codec`). The
Python-side dispatcher logs under `insim_o3.handler` and `insim_o3.client`.

## Running tests

Build the extension first, then run pytest:

```bash
cd insim_o3
uv run maturin develop
uv run pytest
```

Tests use `TestClient` to inject synthetic packets without a real LFS
connection, so no running LFS instance is needed.
