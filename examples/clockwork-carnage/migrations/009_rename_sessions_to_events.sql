-- Primary table
ALTER TABLE sessions RENAME TO events;

-- Auxiliary per-mode tables
ALTER TABLE shortcut_sessions  RENAME TO shortcut_events;

-- FK column renames in child tables
ALTER TABLE shortcut_events    RENAME COLUMN session_id TO event_id;
ALTER TABLE metronome_results  RENAME COLUMN session_id TO event_id;
ALTER TABLE shortcut_times     RENAME COLUMN session_id TO event_id;
ALTER TABLE bomb_runs          RENAME COLUMN session_id TO event_id;
