# Clockwork Carnage
PoC / dog fooding for the insim and kitcar crates

## Modes

### Shortcut

Always-on time attack. Players cross a **Checkpoint 1** object to start a timed attempt, then reach a **Finish** object to record their time. Personal bests are tracked throughout the session. No rounds, no admin interaction required — players can retry immediately without leaving.

**Mapper notes:**
- Place at least one `Checkpoint 1` (start) and one `Finish` object.
- Multiple instances of each are fine — any crossing counts.
- The route between them is entirely up to the layout. Short technical sections, long straights, whatever suits the track.

---

### Metronome

Always-on precision challenge. Players cross a **Checkpoint 1** to start and a **Finish** to stop. The goal is to match a configured target time as closely as possible. Score is the absolute delta from the target; smaller is better.

**Tiers:**

| Tier | Delta |
|------|-------|
| Platinum | ≤ 0.1s |
| Gold | ≤ 0.5s |
| Silver | ≤ 2s |
| Bronze | ≤ 5s |

**Mapper notes:**
- Same object requirements as Shortcut: one or more `Checkpoint 1` (start) and `Finish` objects.
- The target time is set in the event configuration (`target_ms`), not the layout — design the route first, then measure a representative lap and set the target accordingly.
- Works best on routes with a predictable feel (consistent braking points, no huge variance lap-to-lap) so players can dial in their timing.

---

### Bomb

Survival mode. Players hit **Checkpoint 1** objects to keep themselves alive. The timer does not reset — whatever time was left when you hit a checkpoint carries over as your window for the next one. Arrive late and you start the next section already under pressure.

**Refresh checkpoints:** Place **Checkpoint 2**, **Checkpoint 3**, or **Finish** objects at key locations to act as refreshes — hitting one resets the timer to the full base window. Use these sparingly as a reward for surviving a hard section.

**Checkpoint 1 — regular:**
- Timer carries over: `next_window = remaining`
- Arrive with 20s left → you have 20s to reach the next one
- Arrive with 3s left → you have 3s

**Checkpoint 2 / 3 / Finish — refresh:**
- Timer resets to the full base window regardless of remaining time
- Announced to the player in yellow as `REFRESH`

**Scoring:** checkpoint count, with survival time as a tiebreaker. Best run per session is recorded on the leaderboard.

**Mapper notes:**
- The base window (configured as `checkpoint_timeout_secs`) and the spacing between checkpoints together determine difficulty. A rough guide: `base_window ÷ average_time_between_checkpoints ≈ maximum checkpoints a skilled driver can reach`.
- Aim for checkpoint gaps of **3–5 seconds** at push pace. Tighter than that and runs end almost immediately; wider and the timer barely matters.
- Place one or two refresh (`Checkpoint 2/3` or `Finish`) objects after the hardest section of the route. Reaching the refresh should feel like an achievement — a second wind for players who survive.
- A route with no refreshes and generous spacing suits casual play. A route with tight gaps and a single late refresh rewards mastery.

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
