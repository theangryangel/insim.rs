CREATE TABLE eras (
    id          BIGSERIAL PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE events ADD COLUMN era_id BIGINT REFERENCES eras(id) ON DELETE SET NULL;
