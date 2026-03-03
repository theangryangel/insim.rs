# Clockwork Carnage

PoC for kitcar crates.
Currently WIP.

The perfect lap, by any means necessary.
Forget the fastest lap. The only goal here is to hit a precise target time.
Pacing, precision, and punishment. Can you find your rhythm in the middle of the wreckage?

## Modes

### Metronome

Admin-driven rounds-based competition. An admin starts the game with `!start`, players compete over multiple rounds to match a target lap time as closely as possible, and points are awarded based on accuracy. Full contact is permitted.

The event binary uses subcommands to manage configuration via the database.

| Top-level flag | Description   | Default              |
| -------------- | ------------- | -------------------- |
| `--db`         | Database path | clockwork-carnage.db |

#### `add` — Set/replace the active event

```sh
cargo run --bin event -- add --track FE1X --layout CC --rounds 5 --target 20
```

| Flag             | Description                     | Default |
| ---------------- | ------------------------------- | ------- |
| `-t, --track`    | Track (required)                |         |
| `-l, --layout`   | Layout name (required)          |         |
| `-r, --rounds`   | Number of rounds                | 5       |
| `--target`       | Target time in seconds          | 20      |

Ends any previously active event before creating a new one.

#### `list` — Show the active event

```sh
cargo run --bin event -- list
```

#### `run` — Run the event

```sh
cargo run --bin event -- run --addr 127.0.0.1:29999
```

| Flag                | Description               | Default |
| ------------------- | ------------------------- | ------- |
| `-a, --addr`        | Server address (required) |         |
| `-p, --password`    | Admin password            |         |
| `-m, --max-scorers` | Max scorers per round     | 10      |

Reads track, layout, rounds, and target from the active event in the database. Use `add` to configure one first.

Chat commands: `!start`, `!end`, `!echo <msg>`, `!help`, `!quit`

### Shortcut

Always-on mode. Players drop in and compete for the fastest checkpoint-to-finish time. Personal bests are tracked across attempts. No rounds, no admin start required.

The challenge binary uses subcommands to manage configuration via the database.

| Top-level flag | Description   | Default              |
| -------------- | ------------- | -------------------- |
| `--db`         | Database path | clockwork-carnage.db |

#### `add` — Set/replace the active challenge

```sh
cargo run --bin challenge -- add --track FE1X --layout CC
```

| Flag             | Description            |
| ---------------- | ---------------------- |
| `-t, --track`    | Track (required)       |
| `-l, --layout`   | Layout name (required) |

Ends any previously active challenge before creating a new one.

#### `list` — Show the active challenge

```sh
cargo run --bin challenge -- list
```

#### `run` — Run the challenge

```sh
cargo run --bin challenge -- run --addr 127.0.0.1:29999
```

| Flag             | Description               |
| ---------------- | ------------------------- |
| `-a, --addr`     | Server address (required) |
| `-p, --password` | Admin password            |

Reads track and layout from the active challenge in the database. Use `add` to configure one first.

Chat commands: `!help`, `!end`, `!quit`
