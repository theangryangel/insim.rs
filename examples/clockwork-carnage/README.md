# Clockwork Carnage
PoC / dog fooding for the insim and kitcar crates

## Modes

### Metronome

Admin-driven rounds-based competition. An admin starts the game with `!start`, players compete over multiple rounds to match a target lap time as closely as possible, and points are awarded based on accuracy. Full contact is permitted.

### Shortcut

Always-on mode. Players drop in and compete for the fastest checkpoint-to-finish time. Personal bests are tracked across attempts. No rounds, no admin start required.

## Architecture / How to use

A single binary (`clockwork-carnage`) connects to an LFS server via InSim, polls a SQLite database for active sessions, drives gameplay, and serves the web dashboard — all in one process. Migrations run automatically on startup.

The InSim connection and web server are independently optional: include `[insim]` to enable the InSim runner, include `[web]` to enable the web dashboard. At least one must be present.

### Config file quickstart

```sh
cp clockwork-carnage.example.toml clockwork-carnage.toml
# edit clockwork-carnage.toml to fill in your values
cargo run
```

The default config path is `clockwork-carnage.toml` in the current directory. Override with `--config /path/to/config.toml`.

### Minimal config (both components)

```toml
[insim]
addr = "127.0.0.1:29999"

[web]
lfs_client_id     = "your-client-id"
lfs_client_secret = "your-client-secret"
lfs_redirect_uri  = "http://localhost:3000/auth/callback"
```

See `clockwork-carnage.example.toml` for all available options.

### Managing sessions

Use the web dashboard (`http://localhost:3000`) to create, list, start, edit, and cancel sessions.
