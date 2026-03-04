-- Users: one row per LFS username, upserted on every connection.
CREATE TABLE users (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    uname     TEXT    NOT NULL UNIQUE,
    pname     TEXT    NOT NULL DEFAULT '',
    last_seen TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Base table: shared across all modes
CREATE TABLE sessions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    mode       TEXT NOT NULL,
    status     TEXT NOT NULL DEFAULT 'PENDING'
                    CHECK(status IN ('PENDING', 'ACTIVE', 'COMPLETED', 'CANCELLED')),
    track      TEXT NOT NULL,
    layout     TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    ended_at   TEXT
);
CREATE UNIQUE INDEX idx_sessions_active ON sessions(status) WHERE status = 'ACTIVE';

-- Metronome extension
CREATE TABLE metronome_sessions (
    session_id    INTEGER PRIMARY KEY REFERENCES sessions(id),
    rounds        INTEGER NOT NULL,
    target_ms     INTEGER NOT NULL,
    max_scorers   INTEGER NOT NULL DEFAULT 10,
    current_round INTEGER NOT NULL DEFAULT 0
);

-- Shortcut extension
CREATE TABLE shortcut_sessions (
    session_id INTEGER PRIMARY KEY REFERENCES sessions(id)
);

-- Metronome results
CREATE TABLE metronome_results (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  INTEGER NOT NULL REFERENCES sessions(id),
    round       INTEGER NOT NULL,
    user_id     INTEGER NOT NULL REFERENCES users(id),
    delta_ms    INTEGER NOT NULL,
    points      INTEGER NOT NULL,
    recorded_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_metronome_results_session ON metronome_results(session_id);
CREATE INDEX idx_metronome_results_combo ON metronome_results(session_id, round);

-- Shortcut times
CREATE TABLE shortcut_times (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES sessions(id),
    user_id    INTEGER NOT NULL REFERENCES users(id),
    vehicle    TEXT    NOT NULL,
    time_ms    INTEGER NOT NULL,
    set_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_shortcut_times_session ON shortcut_times(session_id);
CREATE INDEX idx_shortcut_times_user ON shortcut_times(session_id, user_id);
