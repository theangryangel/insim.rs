CREATE TABLE player (
    id INTEGER PRIMARY KEY,
    uname TEXT UNIQUE NOT NULL,   -- LFSW username
    pname TEXT NOT NULL,          -- Last playername
    first_seen_at TEXT NOT NULL,  -- jiff::Timestamp serialised
    last_seen_at TEXT NOT NULL    -- jiff::Timestamp serialised
);

CREATE TABLE event (
  id INTEGER PRIMARY KEY,
  started_at TEXT NOT NULL,
  completed_at TEXT,

  -- denormalised combo from config.yaml, used for historical info only
  name TEXT NOT NULL,
  track TEXT NOT NULL,               -- insim::core::Vehicle serialised
  layout TEXT,                       -- e.g., "20s_cup"
  target_time TEXT NOT NULL,         -- jiff::Span serialised (e.g., "20s")
  restart_after TEXT NOT NULL,       -- jiff::Span serialised (e.g., "1m10s")
  rounds INTEGER NOT NULL
);

CREATE TABLE result (
  event_id INTEGER NOT NULL,
  round INTEGER NOT NULL,
  player_id INTEGER NOT NULL,
  position INTEGER NOT NULL,
  points INTEGER NOT NULL,
  delta INTEGER NOT NULL,        -- Duration in milliseconds

  PRIMARY KEY (event_id, round, position),
  FOREIGN KEY (event_id) REFERENCES event(id) ON DELETE CASCADE,
  FOREIGN KEY (player_id) REFERENCES player(id) ON DELETE CASCADE
);

CREATE INDEX idx_result_player ON result(player_id);
