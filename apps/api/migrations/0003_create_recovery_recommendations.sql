-- 0003_create_recovery_recommendations.sql
-- Records optimized recovery recommendations and plan quotes for audit history.
-- A stored recovery recommendation is advisory only and NOT an executed transaction.

CREATE TABLE IF NOT EXISTS recovery_recommendations (
    recovery_id UUID PRIMARY KEY,
    position_id VARCHAR(255) NOT NULL,
    request_params JSONB NOT NULL,
    ranked_response JSONB NOT NULL,
    status VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_recovery_recommendations_position ON recovery_recommendations (position_id);
CREATE INDEX IF NOT EXISTS idx_recovery_recommendations_created ON recovery_recommendations (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_recovery_recommendations_status ON recovery_recommendations (status);

