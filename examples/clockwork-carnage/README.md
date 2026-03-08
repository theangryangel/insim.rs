# Clockwork Carnage
PoC / dog fooding for the insim and kitcar crates

## Modes

### Metronome

Admin-driven rounds-based competition. An admin starts the game with `!start`, players compete over multiple rounds to match a target lap time as closely as possible, and points are awarded based on accuracy. Full contact is permitted.

### Shortcut

Always-on mode. Players drop in and compete for the fastest checkpoint-to-finish time. Personal bests are tracked across attempts. No rounds, no admin start required.

## Architecture / How to use

A single binary (`clockwork-carnage`) connects to an LFS server via InSim, polls a SQLite database for active sessions, drives gameplay, and serves the web dashboard — all in one process. Migrations run automatically on startup.

### Environment variables (required for `run`)

| Variable | Description |
|---|---|
| `LFS_CLIENT_ID` | OAuth2 client ID from id.lfs.net |
| `LFS_CLIENT_SECRET` | OAuth2 client secret |
| `LFS_REDIRECT_URI` | OAuth2 redirect URI (e.g. `http://localhost:3000/auth/callback`) |
| `SESSION_KEY` | 64-byte cookie signing key (defaults to `aaaa...` in dev) |

### Creating sessions

```sh
# Metronome session
cargo run -- add metronome \
  --track BL1 --layout "" --rounds 5 --target 20 --max-scorers 10 \
  --name "Friday Night Carnage" \
  --scheduled-at "2026-03-15 19:00"

# Shortcut session
cargo run -- add shortcut \
  --track AU1 --layout "" \
  --name "AU1 Time Attack" \
  --scheduled-at "2026-03-16 14:00"
```

`--name`, `--description`, and `--scheduled-at` are all optional.

### Running

```sh
cargo run -- run --addr 127.0.0.1:29999 --listen 127.0.0.1:3000
```

`--listen` defaults to `127.0.0.1:3000`.

### Other commands

```sh
# List all sessions
cargo run -- list

# Activate a pending session
cargo run -- activate <id>

# Set a post-event write-up
cargo run -- writeup <id> "Great event, congrats to the winners!"
```
