CREATE TABLE events (
    aggregate_type VARCHAR(255) NOT NULL,
    aggregate_id VARCHAR(255) NOT NULL,
    sequence BIGINT NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    event_version VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (aggregate_type, aggregate_id, sequence)
);

CREATE INDEX IF NOT EXISTS idx_events_aggregate 
ON events (aggregate_type, aggregate_id, sequence ASC);