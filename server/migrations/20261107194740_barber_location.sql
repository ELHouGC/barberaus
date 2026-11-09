-- Enable PostGIS
CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE locations (
    location_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    name TEXT,
    address TEXT,

    location GEOGRAPHY(POINT, 4326) NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_locations_geography
    ON locations USING GIST (location);

CREATE TABLE barber_location_assignments (
    barber_id UUID NOT NULL
        REFERENCES user_credentials(user_id) ON DELETE CASCADE,

    location_id UUID NOT NULL
        REFERENCES locations(location_id) ON DELETE CASCADE,

    is_primary BOOLEAN NOT NULL DEFAULT FALSE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (barber_id, location_id)
);

CREATE INDEX idx_barber_location_assignments_location
    ON barber_location_assignments(location_id);

CREATE UNIQUE INDEX idx_barber_primary_location
    ON barber_location_assignments(barber_id)
    WHERE is_primary = TRUE;