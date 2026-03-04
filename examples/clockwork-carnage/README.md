# Clockwork Carnage
PoC / dog fooding for the insim and kitcar crates

## Modes

### Metronome

Admin-driven rounds-based competition. An admin starts the game with `!start`, players compete over multiple rounds to match a target lap time as closely as possible, and points are awarded based on accuracy. Full contact is permitted.

### Shortcut

Always-on mode. Players drop in and compete for the fastest checkpoint-to-finish time. Personal bests are tracked across attempts. No rounds, no admin start required.

## Architecture / How to use

There are two binaries:

- **`runner`** — connects to an LFS server via InSim, polls the database for active sessions, and drives gameplay.
- **`web`** — read-only web dashboard (default `localhost:3000`) showing upcoming/active sessions, results, and standings.

Both share a SQLite database. Migrations run automatically on startup.

### Creating sessions

```sh
# Metronome session
cargo run --bin runner -- add metronome \
  --track BL1 --layout "" --rounds 5 --target 20 --max-scorers 10 \
  --name "Friday Night Carnage" \
  --scheduled-at "2026-03-15 19:00"

# Shortcut session
cargo run --bin runner -- add shortcut \
  --track AU1 --layout "" \
  --name "AU1 Time Attack" \
  --scheduled-at "2026-03-16 14:00"
```

`--name`, `--description`, and `--scheduled-at` are all optional.

### Running

```sh
# Start the game runner
cargo run --bin runner -- run --addr 127.0.0.1:29999

# Start the web dashboard
cargo run --bin web
```

### Other commands

```sh
# List all sessions
cargo run --bin runner -- list

# Activate a pending session
cargo run --bin runner -- activate <id>

# Set a post-event write-up
cargo run --bin runner -- writeup <id> "Great event, congrats to the winners!"
```
