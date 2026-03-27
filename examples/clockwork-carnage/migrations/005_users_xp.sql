CREATE TABLE users_xp (
    id          INTEGER PRIMARY KEY,
    user_id     INTEGER NOT NULL REFERENCES users(id),
    event_id    INTEGER REFERENCES events(id),
    amount      INTEGER NOT NULL,
    reason      TEXT NOT NULL,
    recorded_at TEXT NOT NULL
);
