# Bomb

Players race against a countdown timer. Hitting checkpoints extends the timer; missing them causes the bomb to explode. Collisions and resets incur time penalties.

## Track and layout setup

The game is configured with a track code and an optional layout:

```
--track BL1 --layout my_layout
```

If no layout is specified the game runs on the bare track with no insim objects, and any circle crossing is accepted in any order.

## Layout objects

The layout uses two kinds of insim objects:

### Insim Circles - ordered checkpoints

Circles define the route players must follow. Each circle has a numeric index (0–255) that determines its position in the sequence.

- The sequence is derived automatically when the layout loads - you do not need to configure it in code.
- Circles are sorted by index, so **index 0 comes first** and acts as the start/finish gate.
- Players must cross circles in strict ascending index order, wrapping back to 0 after the highest index. Crossing a circle out of order is silently ignored.
- **Multiple circles with the same index** are valid alternate paths - the player may cross any one of them to satisfy that step in the sequence. Use this for splits and chicanes where there are two physical routes through the same gate.
- When a run expires (bomb goes off) the player can resume from the next circle they cross - they do not need to pit or spectate.

**Example sequence** for a simple three-gate layout:

```
0 (start/finish) -> 1 -> 2 -> 3 -> 0 (start/finish) -> ...
```

Place each circle at a bottleneck - the top of a jump, a narrow bridge, or a tight chicane - so that it cannot be physically bypassed. The code enforces ordering but cannot enforce coverage of every gate if alternate routes exist that avoid a circle entirely.

### Insim Checkpoints - time bonus gates

Standard LFS insim checkpoints (finish line or CP1/CP2/CP3) act as **time bonus** gates, independent of the circle sequence. Crossing one in either direction resets the player's countdown to the full checkpoint timeout and resets the penalty accumulation window.

- These do not affect the circle sequence or `last_circle_index`.
- Place them at the end of a long straight or after a difficult section as a reward for clean driving.
- Any checkpoint kind (finish, CP1, CP2, CP3) works identically.

## Timer mechanics

| Event                     | Effect                                                                  |
| ------------------------- | ----------------------------------------------------------------------- |
| Circle crossed (in order) | Extends deadline by current window; window shrinks by penalty each time |
| Insim checkpoint crossed  | Resets deadline to full timeout and resets the window                   |
| Collision                 | Reduces deadline proportionally to closing speed                        |
| Car reset                 | Fixed time penalty                                                      |
| Pitting                   | Run ends immediately                                                    |

The shrinking window on circles means players cannot survive indefinitely by farming the same stretch - they must keep moving through new gates or find a time bonus checkpoint.
