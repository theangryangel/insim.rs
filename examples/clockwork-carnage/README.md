# Clockwork Carnage

PoC / dog fooding for the insim and kitcar crates

## Modes

### Shortcut

Always-on time attack. Players cross a **Checkpoint 1** object to start a timed attempt, then reach a **Finish** object to record their time. Personal bests are tracked throughout the session. No rounds, no admin interaction required - players can retry immediately without leaving.

**Mapper notes:**

- Place at least one `Checkpoint 1` (start) and one `Finish` object.
- Multiple instances of each are fine - any crossing counts.
- The route between them is entirely up to the layout. Short technical sections, long straights, whatever suits the track.

---

### Metronome

Always-on precision challenge. Players cross a **Checkpoint 1** to start and a **Finish** to stop. The goal is to match a configured target time as closely as possible. Score is the absolute delta from the target; smaller is better.

**Tiers:**

| Tier     | Delta  |
| -------- | ------ |
| Platinum | ≤ 0.1s |
| Gold     | ≤ 0.5s |
| Silver   | ≤ 2s   |
| Bronze   | ≤ 5s   |

**Mapper notes:**

- Same object requirements as Shortcut: one or more `Checkpoint 1` (start) and `Finish` objects.
- The target time is set in the event configuration (`target_ms`), not the layout - design the route first, then measure a representative lap and set the target accordingly.
- Works best on routes with a predictable feel (consistent braking points, no huge variance lap-to-lap) so players can dial in their timing.

---

### Bomb

Survival mode. Players hit checkpoint objects to stay alive. Every run begins with a full timer window. Each checkpoint costs a small time penalty - the window shrinks permanently as checkpoints accumulate. Reach a **Finish** object to fully reset the window back to the base value.

**Checkpoints (any kind except Finish):**

- Deduct `checkpoint_penalty_ms` from the current window: `next_window = current_window - penalty`
- After enough checkpoints the window becomes very tight, even if you arrive early
- Announced to the player in green as `checkpoint N - -Xs - Ys left`

**Finish - refresh:**

- Resets the window back to the full `checkpoint_timeout_secs` value regardless of current window size
- Announced to the player in yellow as `FINISH - checkpoint N - REFRESHED Ys`

**Car reset penalty:**

- Resetting your car (via the in-game reset or object collision reset) deducts `checkpoint_penalty_ms` directly from the time remaining on the clock - the deadline moves backwards without touching the window size
- Announced to the player in red as `RESET penalty - -Xs - Ys left`

**Pitting ends the run:**

- Driving into the pits during an active run immediately ends the run and specs the player
- The run is recorded with however many checkpoints were reached before pitting
- Players must commit to their fuel load before starting - there is no opportunity to refuel mid-run

**Collision penalty:**

- Car-to-car contact deducts time proportional to the closing speed at impact
- Formula: `penalty = (closing_speed / 30 m/s).clamp(0, 1) × collision_max_penalty_ms`
- A light brush at low closing speed costs little; a head-on ram at ≥ 30 m/s (≈ 108 km/h) deducts the full `collision_max_penalty_ms` - default 500 ms
- Both drivers are penalised independently if they have an active run
- Announced in red as `COLLISION - -Xs - Ys left`

**Scoring:** checkpoint count, with survival time as a tiebreaker. Best run per session is recorded on the leaderboard.

**Mapper notes:**

- `checkpoint_timeout_secs` sets the starting window and what a Finish refresh returns you to. `checkpoint_penalty_ms` controls how fast the window erodes - default is 250 ms per checkpoint. `collision_max_penalty_ms` sets the maximum time deducted for a full-speed collision - default is 500 ms.
- A low penalty with a long timeout rewards checkpoint volume. A high penalty makes early checkpoints cheap but forces a Finish refresh before the window vanishes.
- Place **Finish** objects on an optional detour rather than the main checkpoint line - players must decide whether to gamble time on the diversion or keep grinding checkpoints on the safe path. The further off-route the Finish, the higher the risk/reward.
- Aim for checkpoint gaps of **3–5 seconds** at push pace. Tighter than that and runs end almost immediately; wider and the timer barely matters.

## Architecture / How to use

A single binary (`clockwork-carnage`) connects to an LFS server via InSim, polls a SQLite database for active sessions, drives gameplay, and serves the web dashboard - all in one process. Migrations run automatically on startup.

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
