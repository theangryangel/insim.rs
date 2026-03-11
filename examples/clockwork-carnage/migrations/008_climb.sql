CREATE TABLE climb_sessions (
    session_id INTEGER PRIMARY KEY REFERENCES sessions(id)
);

CREATE TABLE climb_times (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES sessions(id),
    user_id    INTEGER NOT NULL REFERENCES users(id),
    vehicle    TEXT    NOT NULL,
    time_ms    INTEGER NOT NULL,
    set_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_climb_times_session ON climb_times(session_id);
CREATE INDEX idx_climb_times_user    ON climb_times(session_id, user_id);
