use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database record for position_snapshots table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct PositionSnapshotRecord {
    pub id: String,
    pub position_id: String,
    pub owner_address: String,
    pub protocol: String,
    pub network: String,
    pub observation_slot: Option<i64>,
    pub data_source: String,
    pub collateral_raw: String,
    pub collateral_decimals: i16,
    pub debt_raw: String,
    pub debt_decimals: i16,
    pub liquidation_threshold_bps: i32,
    pub snapshot_payload: serde_json::Value,
    pub observed_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// Database record for oracle_observations table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct OracleObservationRecord {
    pub id: i64,
    pub asset_id: String,
    pub oracle_source: String,
    pub price_mantissa: i64,
    pub confidence_mantissa: i64,
    pub exponent: i32,
    pub published_at: DateTime<Utc>,
    pub observed_slot: Option<i64>,
    pub ingested_at: DateTime<Utc>,
}

/// Database record for recovery_recommendations table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct RecoveryRecommendationRecord {
    pub recovery_id: Uuid,
    pub position_id: String,
    pub request_params: serde_json::Value,
    pub ranked_response: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
