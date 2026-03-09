CREATE TABLE bomb_runs (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id        INTEGER NOT NULL REFERENCES sessions(id),
    user_id           INTEGER NOT NULL REFERENCES users(id),
    vehicle           TEXT    NOT NULL,
    checkpoint_count  INTEGER NOT NULL,
    survival_ms       INTEGER NOT NULL,
    recorded_at       TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_bomb_runs_session ON bomb_runs(session_id);
