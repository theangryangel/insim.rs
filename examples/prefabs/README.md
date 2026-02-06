# prefab toolbox

Prefab toolbox for LFS layout editing.

Features:

- Draws an in-game button toolbox with all prefab names.
- Periodically polls current layout selection (`TtcType::Sel`).
- Click prefab button to replace selected object(s) with a clipboard prefab at the anchor position.
- Placement is done via `PMO_SELECTION` clipboard mode only (no `PMO_ADD_OBJECTS`, no `PMO_SELECTION_REAL`).
- Reload prefabs from disk via `Reload YAML` button.
- Save multi-object selection as a new prefab via `Save Selection`.
- Paint ad-hoc text into selection via `Paint Text` + type-in prompt.

## Usage

- `cargo run -- --help`
- Example: `cargo run -- tcp --addr 127.0.0.1:29999 ./prefabs.yaml`
- A starter `prefabs.yaml` is included in this directory.

## YAML schema

Top-level file shape:

```yaml
prefabs:
  - name: "my-prefab"
    objects:
      - !Control
        xyz:
          x: -9556
          y: -30695
          z: 8
        kind: Start
        heading:
          radians: 0.0
        floating: false
```

`objects` entries are direct serde representations of `ObjectInfo`.
