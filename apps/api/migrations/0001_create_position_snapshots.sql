-- 0001_create_position_snapshots.sql
-- Persists normalized lending position snapshots as a cache and audit history.
-- This database is an advisory cache, not the source of truth for on-chain state.

CREATE TABLE IF NOT EXISTS position_snapshots (
    id VARCHAR(255) PRIMARY KEY,
    position_id VARCHAR(255) NOT NULL,
    owner_address VARCHAR(255) NOT NULL,
    protocol VARCHAR(64) NOT NULL,
    network VARCHAR(64) NOT NULL,
    observation_slot BIGINT,
    data_source VARCHAR(64) NOT NULL,
    collateral_raw VARCHAR(64) NOT NULL,
    collateral_decimals SMALLINT NOT NULL,
    debt_raw VARCHAR(64) NOT NULL,
    debt_decimals SMALLINT NOT NULL,
    liquidation_threshold_bps INTEGER NOT NULL,
    snapshot_payload JSONB NOT NULL,
    observed_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_position_snapshots_owner ON position_snapshots (owner_address);
CREATE INDEX IF NOT EXISTS idx_position_snapshots_protocol_network ON position_snapshots (protocol, network);
CREATE INDEX IF NOT EXISTS idx_position_snapshots_position_id ON position_snapshots (position_id);
CREATE INDEX IF NOT EXISTS idx_position_snapshots_observed_at ON position_snapshots (observed_at DESC);

