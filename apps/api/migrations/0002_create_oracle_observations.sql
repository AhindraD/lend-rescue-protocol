-- 0002_create_oracle_observations.sql
-- Records oracle price snapshots with raw integer mantissa and exponent.
-- Floating-point representations are strictly prohibited in the persistence layer.

CREATE TABLE IF NOT EXISTS oracle_observations (
    id BIGSERIAL PRIMARY KEY,
    asset_id VARCHAR(64) NOT NULL,
    oracle_source VARCHAR(64) NOT NULL,
    price_mantissa BIGINT NOT NULL,
    confidence_mantissa BIGINT NOT NULL,
    exponent INTEGER NOT NULL,
    published_at TIMESTAMPTZ NOT NULL,
    observed_slot BIGINT,
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_oracle_observations_asset ON oracle_observations (asset_id, published_at DESC);
CREATE INDEX IF NOT EXISTS idx_oracle_observations_source ON oracle_observations (oracle_source);
CREATE INDEX IF NOT EXISTS idx_oracle_observations_ingested ON oracle_observations (ingested_at DESC);

