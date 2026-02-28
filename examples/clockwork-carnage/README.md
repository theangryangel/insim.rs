# Clockwork Carnage

PoC for kitcar crates.
Currently WIP.

The perfect lap, by any means necessary.
Forget the fastest lap. The only goal here is to hit a precise target time.
Pacing, precision, and punishment. Can you find your rhythm in the middle of the wreckage?

## Modes

### Event

Admin-driven rounds-based competition. An admin starts the game with `!start`, players compete over multiple rounds to match a target lap time as closely as possible, and points are awarded based on accuracy. Full contact is permitted.

```sh
cargo run --bin event -- --addr 127.0.0.1:29999
```

| Flag                | Description               | Default |
| ------------------- | ------------------------- | ------- |
| `-a, --addr`        | Server address (required) |         |
| `-p, --password`    | Admin password            | none    |
| `-r, --rounds`      | Number of rounds          | 5       |
| `-m, --max-scorers` | Max scorers per round     | 10      |
| `-t, --track`       | Track                     | FE1X    |
| `-l, --layout`      | Layout name               | CC      |

Chat commands: `!start`, `!end`, `!echo <msg>`, `!help`, `!quit`

### Challenge

Always-on mode. Players drop in and compete for the fastest checkpoint-to-finish time. Personal bests are tracked across attempts. No rounds, no admin start required.

```sh
cargo run --bin challenge -- --addr 127.0.0.1:29999
```

| Flag             | Description               | Default |
| ---------------- | ------------------------- | ------- |
| `-a, --addr`     | Server address (required) |         |
| `-p, --password` | Admin password            | none    |
| `-t, --track`    | Track                     | FE1X    |
| `-l, --layout`   | Layout name               | CC      |

Chat commands: `!help`, `!end`, `!quit`
