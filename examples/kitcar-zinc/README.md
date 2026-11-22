# kitcar-zinc

ZINC Is Not CTRA: a small tribute to the community-run LFS network whose points and licence system made clean public racing work. Not affiliated.

- Races rotate between BL1 and FE1, two races each. Finished races restart on their own, 60 seconds after the winner.
- A confirmed, clean result earns a point. Yellow flags are counted.
- Each point unlocks the next cars, and you keep them:

| Points | Unlocks             |
| ------ | ------------------- |
| 0      | UF1                 |
| 1      | XFR                 |
| 2      | XRG                 |
| 3      | XRT, RB4, FXO (TBO) |
| 4      | LX4                 |

Two services run side by side: a `RaceDirector` for the rotation, and ZINC's rules as a `Router`. Everything is in memory, so it resets on restart.

## Usage

kitcar is for multiplayer servers, so host a server (or use a dedicated host) first.

- Type `/insim 29999` in chat to start InSim.
- `cargo run -p kitcar-zinc -- --addr 127.0.0.1:29999`
- Add `--admin-password <password>` if your host has one. The director needs it to change track.
- Join, type `/ready` in the lobby, and race.
- `!stats` shows your points, yellows and cars. `!top` shows the leaders. `!rank` opens a card with your standing.
