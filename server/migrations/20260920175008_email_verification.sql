-- Add migration script here
CREATE TABLE email_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    user_id UUID NOT NULL
        REFERENCES user_credentials(user_id)
        ON DELETE CASCADE,

    token_hash TEXT NOT NULL,

    expires_at TIMESTAMPTZ NOT NULL,

    verified_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Prevent duplicate active tokens for the same user, not globally
    CONSTRAINT uq_user_token_hash UNIQUE (user_id, token_hash)
);

-- Index for lookup by user_id and token_hash (used in verify())
CREATE INDEX idx_email_verifications_user_token 
    ON email_verifications(user_id, token_hash);

-- Index for pending verification lookup (used in prepare_for_verification())
CREATE INDEX idx_email_verifications_pending 
    ON email_verifications(user_id, created_at DESC) 
    WHERE verified_at IS NULL;