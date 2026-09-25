-- Migration: Create unified user_views table and JSONB indexes for cqrs-es / postgres-es

-- 1. Unified Views Table
CREATE TABLE IF NOT EXISTS user_views (
    view_id VARCHAR(255) PRIMARY KEY,
    version BIGINT NOT NULL DEFAULT 0,
    payload JSONB NOT NULL,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. Identity & Authentication Indexes
-- Fast lookup by email address (auth flows)
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_views_identity_email 
ON user_views ((payload->'identity'->>'email')) 
WHERE payload->'identity'->>'email' IS NOT NULL;

-- Filter verified/unverified users
CREATE INDEX IF NOT EXISTS idx_user_views_identity_verified 
ON user_views (((payload->'identity'->>'is_email_verified')::boolean));

-- 3. Client Profile Query Indexes
-- Filter client views by city for local feed/barber discovery
CREATE INDEX IF NOT EXISTS idx_user_views_client_city 
ON user_views ((payload->'client_profile'->>'city')) 
WHERE payload->'client_profile' IS NOT NULL;

-- GIN index on selected styles array for fast overlap matching (&>)
CREATE INDEX IF NOT EXISTS idx_user_views_client_styles 
ON user_views USING GIN ((payload->'client_profile'->'selected_styles')) 
WHERE payload->'client_profile' IS NOT NULL;

-- GIN index on followed barbers array
CREATE INDEX IF NOT EXISTS idx_user_views_client_followed_barbers 
ON user_views USING GIN ((payload->'client_profile'->'followed_barber_ids')) 
WHERE payload->'client_profile' IS NOT NULL;

-- 4. Barber Profile Query Indexes
-- Lookup barber by city
CREATE INDEX IF NOT EXISTS idx_user_views_barber_city 
ON user_views ((payload->'barber_profile'->>'city')) 
WHERE payload->'barber_profile' IS NOT NULL;

-- Sort/filter barbers by follower count or ratings
CREATE INDEX IF NOT EXISTS idx_user_views_barber_followers 
ON user_views ((((payload->'barber_profile'->>'follower_count')::int))) 
WHERE payload->'barber_profile' IS NOT NULL;

-- Trigger to automatically touch last_updated on view mutation
CREATE OR REPLACE FUNCTION update_user_views_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_updated = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_user_views_timestamp
BEFORE UPDATE ON user_views
FOR EACH ROW
EXECUTE FUNCTION update_user_views_timestamp();