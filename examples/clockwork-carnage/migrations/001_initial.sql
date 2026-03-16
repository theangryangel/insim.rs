-- Users: one row per LFS username, upserted on every connection.
CREATE TABLE users (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    uname              TEXT    NOT NULL UNIQUE,
    pname              TEXT    NOT NULL DEFAULT '',
    last_seen          TEXT    NOT NULL DEFAULT (datetime('now')),
    oauth_access_token TEXT,
    admin              BOOLEAN NOT NULL DEFAULT FALSE
);

-- Base event table: shared across all modes.
-- `mode` holds a JSON object with a `type` discriminant.
CREATE TABLE events (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    mode             TEXT    NOT NULL,
    status           TEXT    NOT NULL DEFAULT 'PENDING'
                             CHECK(status IN ('PENDING', 'ACTIVE', 'COMPLETED', 'CANCELLED')),
    track            TEXT    NOT NULL,
    layout           TEXT    NOT NULL DEFAULT '',
    created_at       TEXT    NOT NULL DEFAULT (datetime('now')),
    started_at       TEXT,
    ended_at         TEXT,
    name             TEXT,
    description      TEXT,
    writeup          TEXT,
    scheduled_at     TEXT,
    scheduled_end_at TEXT
);
CREATE UNIQUE INDEX idx_events_active ON events(status) WHERE status = 'ACTIVE';

-- Metronome results: one row per lap attempt.
CREATE TABLE metronome_results (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id    INTEGER NOT NULL REFERENCES events(id),
    user_id     INTEGER NOT NULL REFERENCES users(id),
    delta_ms    INTEGER NOT NULL,
    recorded_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_metronome_results_event ON metronome_results(event_id);

-- Shortcut times: one row per timed run.
CREATE TABLE shortcut_times (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL REFERENCES events(id),
    user_id  INTEGER NOT NULL REFERENCES users(id),
    vehicle  TEXT    NOT NULL,
    time_ms  INTEGER NOT NULL,
    set_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_shortcut_times_event ON shortcut_times(event_id);
CREATE INDEX idx_shortcut_times_user  ON shortcut_times(event_id, user_id);

-- Bomb runs: one row per run attempt.
CREATE TABLE bomb_runs (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id         INTEGER NOT NULL REFERENCES events(id),
    user_id          INTEGER NOT NULL REFERENCES users(id),
    vehicle          TEXT    NOT NULL,
    checkpoint_count INTEGER NOT NULL,
    survival_ms      INTEGER NOT NULL,
    recorded_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_bomb_runs_event ON bomb_runs(event_id);
