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

The two main classes are `App` (composition root - holds config, handlers, and
middleware) and `Connection` (thin send-only wrapper passed to every handler).

### Standalone bot

```python
from insim_o3 import App, Connection
from insim_o3.packets import IsiFlag, Mso, Ncn

app = App(flags=[IsiFlag.MSO_COLS])

@app.on
async def on_join(packet: Ncn, conn: Connection) -> None:
    print(f"[join] {packet.pname} ({packet.uname})")

@app.on
async def on_chat(packet: Mso, conn: Connection) -> None:
    print(f"[chat] {packet.msg}")

app.run("127.0.0.1:29999")
```

`run()` connects, dispatches until the connection drops or Ctrl+C, and tears
down cleanly.

### Embedding in another asyncio program

When you need to run alongside other async tasks (a web server, another
client, etc.), use `app.connect()` as an async context manager:

```python
import asyncio
from insim_o3 import App, Connection
from insim_o3.packets import Ncn

app = App()

@app.on
async def on_join(packet: Ncn, conn: Connection) -> None: ...

async def main() -> None:
    async with app.connect("127.0.0.1:29999"):
        await app.serve()

asyncio.run(main())
```

### Connection options

All connection parameters are passed to `App`; only the address goes to
`run()` / `connect()`:

```python
from insim_o3 import App
from insim_o3.packets import IsiFlag

app = App(
    flags=[IsiFlag.MCI, IsiFlag.MSO_COLS],
    iname="my_app",
    admin_password="secret",
    interval_ms=500,
    prefix="!",      # route !-prefixed chat as Mso with usertype=Prefix
    capacity=256,    # broadcast channel buffer size
)
app.run("127.0.0.1:29999")
```

### Sending packets

Every handler receives a `Connection` as its second argument. Use it to send:

```python
from insim_o3 import Connection
from insim_o3.packets import Mso, Mtc, SoundType
from insim_o3.handler import on

@on
async def reply(self, packet: Mso, conn: Connection) -> None:
    await conn.send(Mtc(
        reqi=0, ucid=packet.ucid, plid=packet.plid,
        sound=SoundType.SysMessage, text="hello",
    ))
```

`Connection` also has convenience methods:

```python
await conn.send_command("/track BL1")          # Mst
await conn.send_message("hello")               # Msx (broadcast)
await conn.send_message("hi", ucid=packet.ucid)  # Mtc (targeted)
```

### Reusable handlers

Group related handlers into a `Handler` subclass. Packet types are inferred
from the first parameter annotation; `|` unions route one method to multiple
types:

```python
from insim_o3 import App, Connection, Handler, on
from insim_o3.packets import Cnl, Ncn

class SessionTracker(Handler):
    @on
    async def join(self, packet: Ncn, conn: Connection) -> None:
        print(f"[+] {packet.pname}")

    @on
    async def leave(self, packet: Cnl, conn: Connection) -> None:
        print(f"[-] ucid={packet.ucid}")

app = App()
app.handlers.add(SessionTracker())
app.run("127.0.0.1:29999")
```

`app.handlers` is a small snapshotting registry: `add(h)` and `remove(h)`
work directly and are safe to call while packets are being dispatched.

### Middleware

Middleware intercepts every packet before handlers run and can emit additional
synthetic events into the dispatch pipeline. Implement the `Middleware`
protocol - three async methods, all required:

```python
from insim_o3 import Connection, Middleware
from insim_o3.dispatcher import AnyPacket

class Logger:
    async def on_connect(self, conn: Connection) -> None:
        print("connected")

    async def on_packet(self, packet: AnyPacket) -> list:
        print(f"<- {type(packet).__name__}")
        return []   # no synthetic events

    async def on_shutdown(self) -> None:
        print("disconnected")

app = App(middleware=[Logger()])
app.run("127.0.0.1:29999")
```

`on_packet` returns a list of additional events to inject. Those events are
dispatched after the real packet and can be any object your handlers are
registered for (including custom dataclasses).

#### PresenceMiddleware

The built-in `PresenceMiddleware` tracks connections and players, and emits
typed synthetic events (`Connected`, `Disconnected`, `PlayerJoined`,
`PlayerLeft`, `Renamed`, `TakingOver`):

```python
from insim_o3 import App, Connection, Handler, on
from insim_o3.presence import Connected, PlayerJoined, PresenceMiddleware

presence = PresenceMiddleware()

class Bot(Handler):
    @on
    async def greet(self, event: Connected, conn: Connection) -> None:
        total = len(presence.connections)
        print(f"[+] {event.conn.uname} - {total} online")

    @on
    async def track(self, event: PlayerJoined, conn: Connection) -> None:
        print(f"[track+] {event.player.pname} in {event.player.vehicle}")

app = App(middleware=[presence], handlers=[Bot()])
app.run("127.0.0.1:29999")
```

Query live state at any time via `presence.connections` (dict keyed by ucid)
and `presence.players` (dict keyed by plid).

### Error handling

Exceptions raised by individual handlers or middleware are caught, logged, and
skipped so one bad callback does not kill the dispatch loop. Override the
policy with the `on_error` kwarg on `App` (and `Handler`):

```python
def reraise(exc: BaseException, packet: object, fn: object) -> None:
    raise exc

app = App(on_error=reraise)
```

The default writes via the standard `logging` module under the
`insim_o3.handler` logger.

### Logging

The Rust core emits `tracing` events forwarded into Python's `logging` module
via `pyo3-log`. Configure them like any other Python logger:

```python
import logging

logging.basicConfig(level=logging.INFO)

# Quiet the noisy packet-decode log if you don't need it:
logging.getLogger("insim.net.codec").setLevel(logging.WARNING)
```

Logger names mirror the Rust module path (e.g. `insim.net.codec`). The
Python-side dispatcher logs under `insim_o3.handler`.

## Running tests

Build the extension first, then run pytest:

```bash
cd insim_o3
uv run maturin develop
uv run pytest
```

Tests use `TestApp` and `MockConnection` to inject synthetic packets without a
real LFS connection, so no running LFS instance is needed:

```python
from insim_o3.test_app import TestApp, MockConnection

async def test_something() -> None:
    conn = MockConnection()
    app = TestApp(conn=conn)
    app.handlers.add(MyBot())
    await app.inject(raw_json)
    assert conn.sent[0].text == "expected reply"
```
