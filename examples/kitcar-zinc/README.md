# kitcar-zinc

ZINC Is Not CTRA: a small tribute to the community-run LFS network whose points and licence system made clean public racing work. Not affiliated.

- Races rotate, reshuffled each pass, through KY1X with the "Mini Oval" layout (10 laps, UF1 only), SO6X with the "Park Life Circuit V1" layout, FE1 (3 laps with a pit stop and a reversed grid, at midnight, midday, or an evening race into the dark, picked each visit), AS1 on live time with a random grid, BL1 at sunrise, and BL3 rallycross with a random grid. The rest are one lap. FE1 allows the UF1, XFG and 1DFF72, BL3 the UF1 and 1DFF72, and AS1 and BL1 every car on the licence ladder. Finished races restart on their own, 60 seconds after the winner.
- A confirmed, clean result earns a point. Yellow flags and collisions are counted, but only while you're racing.
- A message of the day greets everyone as they connect, and stays until they close it.
- Each point unlocks the next cars, and you keep them. ZINC checks your licence as you join the track, so locked cars are turned away:

| Points | Unlocks             |
| ------ | ------------------- |
| 0      | UF1, XFG            |
| 1      | 1DFF72 (a mod)      |
| 2      | XFR                 |
| 3      | XRG                 |
| 4      | XRT, RB4, FXO (TBO) |
| 5      | LX4                 |

The host's `RaceDirector` runs the rotation, and ZINC's rules run as a `Router` that reads it for the topbar. Everything is in memory, so it resets on restart.

## Usage

kitcar is for multiplayer servers, so host a server (or use a dedicated host) first.

- Type `/insim 29999` in chat to start InSim.
- `cargo run -p kitcar-zinc -- --addr 127.0.0.1:29999`
- Add `--admin-password <password>` if your host has one. The director needs it to change track.
- ZINC answers LFS's join requests, and LFS waits for its answer, so joins stall if ZINC stops responding.
- Join, type `/ready` in the lobby, and race.
- `!stats` shows your points, yellows and cars. `!top` shows the leaders. `!rank` opens a card with your standing.
