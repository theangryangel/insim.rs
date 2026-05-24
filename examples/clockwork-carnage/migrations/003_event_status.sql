ALTER TABLE events
    ADD COLUMN status TEXT NOT NULL DEFAULT 'pending'
        CONSTRAINT events_status_check CHECK (status IN ('pending', 'live', 'completed'));

UPDATE events
SET status = CASE
    WHEN ended_at IS NOT NULL THEN 'completed'
    WHEN scheduled_at IS NULL OR scheduled_at <= NOW() THEN 'live'
    ELSE 'pending'
END;
