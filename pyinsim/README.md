# pyinsim

Python bindings for [insim.rs](https://github.com/theangryangel/insim.rs), built with [PyO3](https://pyo3.rs) and [maturin](https://maturin.rs).

## Development setup

Install dependencies and build the native extension in-place:

```bash
cd pyinsim
uv sync --dev
uv run maturin develop
```

`maturin develop` compiles the Rust extension and installs it into the virtual
environment created by `uv sync`. Re-run it any time you change Rust source
files under `src/`.

## Packet type codegen

`pyinsim/insim_schema.json` and `python/insim_rs/_types.py` are auto-generated
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

Create an `Insim`, register typed handlers with `@client.on`, then call
`client.run()` to block and process packets:

```python
from insim_rs import Insim
from insim_rs._types import Ncn, Mso

client = Insim("127.0.0.1:29999")

@client.on(Ncn)
def on_join(packet: Ncn) -> None:
    print(f"[join] {packet.pname} ({packet.uname})")

@client.on(Mso)
def on_chat(packet: Mso) -> None:
    print(f"[chat] {packet.msg}")

client.run()
```

Press Ctrl+C to disconnect cleanly.

### Connection options

```python
from insim_rs import Insim
from insim_rs._types import IsiFlag

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
from insim_rs._types import Mtc, SoundType

client.send(Mtc(ucid=packet.ucid, plid=packet.plid, sound=SoundType.SysMessage, text="hello"))
```

### Reusable handlers

Group related handlers into a `Handler` and attach it to a client:

```python
from insim_rs import Insim
from insim_rs.handler import Handler
from insim_rs._types import Ncn

joins = Handler()

@joins.on(Ncn)
def on_join(packet: Ncn) -> None:
    print(f"[join] {packet.pname}")

client = Insim("127.0.0.1:29999")
client.include_handler(joins)
client.run()
```

### Middleware

Middleware runs before handlers and receives every packet regardless of type —
useful for logging or metrics:

```python
from insim_rs import Insim
from insim_rs.dispatcher import AnyPacket

client = Insim("127.0.0.1:29999")

@client.middleware
def log_all(packet_type: str, packet: AnyPacket) -> None:
    print(f"← {packet_type}")

client.run()
```

## Running tests

Build the extension first, then run pytest:

```bash
cd pyinsim
uv run maturin develop
uv run pytest
```

Tests use `TestClient` to inject synthetic packets without a real LFS
connection, so no running LFS instance is needed.
