# prefab toolbox

Prefab toolbox for Live for Speed layout editing via InSim.

Features:

- Spawn prefabs at the current selection anchor (first selected object).
- Save the current multi-object selection to YAML as a new prefab.
- Reload prefab definitions from disk.
- Paint ad-hoc text using painted letter objects.
- Distribute objects evenly along a spline (read: curve) defined by the current selection.

## Usage

- `cargo run -- --help`
- Example: `cargo run -- --addr 127.0.0.1:29999 --prefabs ./prefabs.yaml`
- A starter `prefabs.yaml` is included in this directory.

### Tabs

- Prefabs: reload YAML, save selection, spawn prefabs (replaces selection).
  - The first selected object is treated as the anchor for placement.
  - For floating items you must have an anchor to work reliably.
- Tools:
  - Paint Text: type a string to paint letters starting at the selection anchor.
  - Spline Distribution: distribute objects along the selection path in a curve (or straight line).
    - First object defines direction; ensure it points to the next object.
    - Select in the order you want the spline to travel; multi-selection may not preserve order.
    - Distribution requires at least two selected objects and a positive spacing value.

## Prefab YAML schema

```yaml
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

`objects` entries are direct serde representations of `ObjectInfo`. The x,y,z
coordinates should be relative to each other.
