-- Add migration script here
-- Demographics Index (Gender filtering)
CREATE INDEX IF NOT EXISTS idx_user_views_identity_gender 
ON user_views ((payload->'identity'->>'gender')) 
WHERE payload->'identity'->>'gender' IS NOT NULL;