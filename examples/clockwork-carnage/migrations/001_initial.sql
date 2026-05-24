CREATE TABLE users (
    id                 BIGSERIAL PRIMARY KEY,
    uname              TEXT      NOT NULL UNIQUE,
    pname              TEXT      NOT NULL DEFAULT '',
    last_seen          TIMESTAMPTZ NOT NULL,
    oauth_access_token TEXT,
    admin              BOOLEAN   NOT NULL DEFAULT FALSE,
    twitch_username    TEXT,
    youtube_username   TEXT
);

CREATE TABLE events (
    id           BIGSERIAL PRIMARY KEY,
    mode         JSONB     NOT NULL,
    track        TEXT      NOT NULL,
    layout       TEXT      NOT NULL DEFAULT '',
    created_at   TIMESTAMPTZ NOT NULL,
    ended_at     TIMESTAMPTZ,
    name         TEXT,
    description  TEXT,
    writeup      TEXT,
    scheduled_at TIMESTAMPTZ,
    allowed_vehicles JSONB NOT NULL DEFAULT '[]'
);

CREATE TABLE metronome_results (
    id          BIGSERIAL PRIMARY KEY,
    event_id    BIGINT NOT NULL REFERENCES events(id),
    user_id     BIGINT NOT NULL REFERENCES users(id),
    delta_ms    BIGINT NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_metronome_results_event ON metronome_results(event_id);

CREATE TABLE shortcut_times (
    id       BIGSERIAL PRIMARY KEY,
    event_id BIGINT NOT NULL REFERENCES events(id),
    user_id  BIGINT NOT NULL REFERENCES users(id),
    vehicle  TEXT   NOT NULL,
    time_ms  BIGINT NOT NULL,
    set_at   TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_shortcut_times_event ON shortcut_times(event_id);
CREATE INDEX idx_shortcut_times_user  ON shortcut_times(event_id, user_id);

CREATE TABLE bomb_runs (
    id               BIGSERIAL PRIMARY KEY,
    event_id         BIGINT NOT NULL REFERENCES events(id),
    user_id          BIGINT NOT NULL REFERENCES users(id),
    vehicle          TEXT   NOT NULL,
    checkpoint_count BIGINT NOT NULL,
    survival_ms      BIGINT NOT NULL,
    recorded_at      TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_bomb_runs_event ON bomb_runs(event_id);
