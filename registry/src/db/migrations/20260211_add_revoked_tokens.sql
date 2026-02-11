CREATE TABLE IF NOT EXISTS revoked_tokens (
    jti UUID PRIMARY KEY,
    expires_at BIGINT NOT NULL
);

-- We only need to keep revoked tokens until they expire naturally.
-- A scheduled job could clean this up, but for now we just store them.
CREATE INDEX idx_revoked_tokens_expires_at ON revoked_tokens(expires_at);
