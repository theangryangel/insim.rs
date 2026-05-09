# strobe

Strobes the lights on your local car in a configurable sequence. Each step in
the sequence sets a combination of [`LclFlags`] (signals, headlights, fog
lights) and holds them for a given duration, then the sequence loops
indefinitely.

The strobe only runs while a local human player is on track. It pauses and
resets to the beginning of the sequence whenever the player pits or leaves
the session, resuming from step one when they return.

## Usage

```
strobe <addr> <file>
```

- `addr` - `host:port` of the LFS InSim interface (e.g. `127.0.0.1:29999`)
- `file` - path to a YAML sequence file (see below)

## Sequence file format

The YAML file contains a `steps` list. Each step has:

| field         | type    | description                                 |
| ------------- | ------- | ------------------------------------------- |
| `duration_ms` | integer | how long to hold this step, in milliseconds |
| `flags`       | string  | `\|`-separated [`LclFlags`] names to apply  |

Available flag names:

| category | flags                                                        |
| -------- | ------------------------------------------------------------ |
| signals  | `SIGNAL_OFF`, `SIGNAL_LEFT`, `SIGNAL_RIGHT`, `SIGNAL_HAZARD` |
| lights   | `LIGHT_OFF`, `LIGHT_SIDE`, `LIGHT_LOW`, `LIGHT_HIGH`         |
| fog      | `FOG_FRONT_OFF`, `FOG_FRONT`, `FOG_REAR_OFF`, `FOG_REAR`     |
| extra    | `EXTRA_OFF`, `EXTRA`                                         |

## Example

You can find a number of examples in this directory.

Run it with:

```
cargo run -p strobe -- 127.0.0.1:29999 examples/strobe/left-to-right.example.yaml
```
