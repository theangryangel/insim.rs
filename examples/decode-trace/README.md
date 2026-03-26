# decode-trace

Decode a raw insim packet from hex bytes and print a field-level trace of the decode process.

## Usage

```
cargo run -p decode-trace -- <hex bytes...>
```

Bytes can be space-separated or run together:

```
cargo run -p decode-trace -- 01 03 02 03
cargo run -p decode-trace -- 01030203
```

## Input format

The bytes are the full on-wire packet including the leading size byte:

| Offset | Content                  |
| ------ | ------------------------ |
| 0      | Size (total bytes / 4)   |
| 1      | Packet type discriminant |
| 2      | ReqI                     |
| 3+     | Payload                  |

## Example output

```
--- input (4 bytes): [01, 03, 02, 03]
--- decode trace:
TRACE decode{field="discriminator"}: insim_core::decode::context: ok bytes=b"\x03"
TRACE decode{field="Tiny::reqi"}:decode{field="val"}: insim_core::decode::context: ok bytes=b"\x02"
TRACE decode{field="Tiny::reqi"}: insim_core::decode::context: ok bytes=b"\x02"
TRACE decode{field="Tiny::subt"}:decode{field="discriminant"}: insim_core::decode::context: ok bytes=b"\x03"
TRACE decode{field="Tiny::subt"}: insim_core::decode::context: ok bytes=b"\x03"
--- result: Tiny(
    Tiny {
        reqi: RequestId(2),
        subt: Ping,
    },
)
```

Each trace line shows the nested span chain of field names and the raw bytes consumed for that field.

## Narrowing output with RUST_LOG

By default only `insim_core` is set to `TRACE`. Use `RUST_LOG` to adjust:

```
# Also trace the insim crate itself
RUST_LOG=insim_core=trace,insim=trace cargo run -p decode-trace -- 01 03 02 03
```
