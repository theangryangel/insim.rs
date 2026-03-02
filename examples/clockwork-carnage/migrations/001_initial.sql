-- Users: one row per LFS username, upserted on every connection.
CREATE TABLE users (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    uname     TEXT    NOT NULL UNIQUE,
    pname     TEXT    NOT NULL DEFAULT '',
    last_seen TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- A challenge is a track+layout combo that players compete on.
-- One challenge is "active" at a time; the binary creates/reuses it on startup.
CREATE TABLE challenges (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    track      TEXT    NOT NULL,
    layout     TEXT    NOT NULL DEFAULT '',
    started_at TEXT    NOT NULL DEFAULT (datetime('now')),
    ended_at   TEXT
);
CREATE INDEX idx_challenges_active ON challenges(ended_at);

-- Every timed run in a challenge. PBs derived via MIN(time_ms) GROUP BY user_id.
CREATE TABLE challenge_times (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    challenge_id INTEGER NOT NULL REFERENCES challenges(id),
    user_id      INTEGER NOT NULL REFERENCES users(id),
    vehicle      TEXT    NOT NULL,
    time_ms      INTEGER NOT NULL,
    set_at       TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_challenge_times_challenge ON challenge_times(challenge_id);
CREATE INDEX idx_challenge_times_user ON challenge_times(challenge_id, user_id);

-- A scheduled event session with N rounds at a target time.
CREATE TABLE events (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    track      TEXT    NOT NULL,
    layout     TEXT    NOT NULL DEFAULT '',
    rounds     INTEGER NOT NULL,
    target_ms  INTEGER NOT NULL,
    started_at TEXT    NOT NULL DEFAULT (datetime('now')),
    ended_at   TEXT
);

-- Per-round results for each player in an event.
CREATE TABLE event_round_results (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id    INTEGER NOT NULL REFERENCES events(id),
    round       INTEGER NOT NULL,
    user_id     INTEGER NOT NULL REFERENCES users(id),
    delta_ms    INTEGER NOT NULL,
    points      INTEGER NOT NULL,
    recorded_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_event_round_event ON event_round_results(event_id);
CREATE INDEX idx_event_round_combo ON event_round_results(event_id, round);
