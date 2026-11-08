use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use domain_types::{PositionSnapshot, Price};
use serde_json::Value;
use sqlx::Row;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::ApiError;
use crate::persistence::models::RecoveryRecommendationRecord;
use crate::persistence::postgres::DatabasePool;

/// Repository for caching and retrieving position snapshots.
#[derive(Debug, Clone)]
pub struct SnapshotRepository {
    pool: DatabasePool,
    memory_cache: Arc<RwLock<HashMap<String, PositionSnapshot>>>,
}

impl SnapshotRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self {
            pool,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn save_snapshot(&self, snapshot: &PositionSnapshot) -> Result<(), ApiError> {
        // Always update memory cache
        self.memory_cache
            .write()
            .await
            .insert(snapshot.id.clone(), snapshot.clone());

        if let Some(pg) = self.pool.pool() {
            let now = Utc::now();
            let expires_at = now + Duration::seconds(300); // 5 minute cache
            let payload = serde_json::to_value(snapshot)
                .map_err(|e| ApiError::Internal(format!("Serialization error: {}", e)))?;

            let _ = sqlx::query(
                r#"
                INSERT INTO position_snapshots (
                    id, position_id, owner_address, protocol, network,
                    observation_slot, data_source, collateral_raw, collateral_decimals,
                    debt_raw, debt_decimals, liquidation_threshold_bps, snapshot_payload,
                    observed_at, expires_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                ON CONFLICT (id) DO UPDATE SET
                    collateral_raw = EXCLUDED.collateral_raw,
                    debt_raw = EXCLUDED.debt_raw,
                    snapshot_payload = EXCLUDED.snapshot_payload,
                    observed_at = EXCLUDED.observed_at,
                    updated_at = EXCLUDED.updated_at
                "#,
            )
            .bind(&snapshot.id)
            .bind(&snapshot.id)
            .bind(&snapshot.owner)
            .bind(format!("{:?}", snapshot.protocol_source))
            .bind("devnet")
            .bind(None as Option<i64>)
            .bind(snapshot.protocol_source.label())
            .bind(snapshot.collateral_amount.raw.to_string())
            .bind(snapshot.collateral_amount.decimals as i16)
            .bind(snapshot.debt_amount.raw.to_string())
            .bind(snapshot.debt_amount.decimals as i16)
            .bind(snapshot.liquidation_threshold_bps as i32)
            .bind(payload)
            .bind(now)
            .bind(Some(expires_at))
            .bind(now)
            .execute(pg)
            .await;
        }

        Ok(())
    }

    pub async fn get_snapshot(&self, id: &str) -> Result<Option<PositionSnapshot>, ApiError> {
        if let Some(snap) = self.memory_cache.read().await.get(id).cloned() {
            return Ok(Some(snap));
        }

        if let Some(pg) = self.pool.pool() {
            let row = sqlx::query(
                "SELECT snapshot_payload FROM position_snapshots WHERE id = $1 LIMIT 1",
            )
            .bind(id)
            .fetch_optional(pg)
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?;

            if let Some(r) = row {
                let payload: serde_json::Value = r
                    .try_get("snapshot_payload")
                    .map_err(|e| ApiError::Database(e.to_string()))?;
                let snap: PositionSnapshot = serde_json::from_value(payload)
                    .map_err(|e| ApiError::Internal(format!("Deserialization error: {}", e)))?;
                return Ok(Some(snap));
            }
        }

        Ok(None)
    }
}

/// Repository for persisting and retrieving oracle observations.
#[derive(Debug, Clone)]
pub struct ObservationRepository {
    pool: DatabasePool,
    memory_cache: Arc<RwLock<HashMap<String, Price>>>,
}

impl ObservationRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self {
            pool,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn save_observation(
        &self,
        asset_id: &str,
        oracle_source: &str,
        price: &Price,
    ) -> Result<(), ApiError> {
        self.memory_cache
            .write()
            .await
            .insert(asset_id.to_string(), price.clone());

        if let Some(pg) = self.pool.pool() {
            let pub_at =
                chrono::DateTime::from_timestamp(price.timestamp, 0).unwrap_or_else(Utc::now);

            let _ = sqlx::query(
                r#"
                INSERT INTO oracle_observations (
                    asset_id, oracle_source, price_mantissa, confidence_mantissa,
                    exponent, published_at, observed_slot
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(asset_id)
            .bind(oracle_source)
            .bind(price.mantissa)
            .bind(price.confidence as i64)
            .bind(price.exponent)
            .bind(pub_at)
            .bind(Some(price.slot as i64))
            .execute(pg)
            .await;
        }

        Ok(())
    }

    pub async fn get_latest_observation(&self, asset_id: &str) -> Result<Option<Price>, ApiError> {
        if let Some(p) = self.memory_cache.read().await.get(asset_id).cloned() {
            return Ok(Some(p));
        }

        if let Some(pg) = self.pool.pool() {
            let row = sqlx::query(
                r#"
                SELECT price_mantissa, confidence_mantissa, exponent, published_at, observed_slot
                FROM oracle_observations
                WHERE asset_id = $1
                ORDER BY published_at DESC
                LIMIT 1
                "#,
            )
            .bind(asset_id)
            .fetch_optional(pg)
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?;

            if let Some(r) = row {
                let price_mantissa: i64 = r
                    .try_get("price_mantissa")
                    .map_err(|e| ApiError::Database(e.to_string()))?;
                let confidence_mantissa: i64 = r
                    .try_get("confidence_mantissa")
                    .map_err(|e| ApiError::Database(e.to_string()))?;
                let exponent: i32 = r
                    .try_get("exponent")
                    .map_err(|e| ApiError::Database(e.to_string()))?;
                let published_at: DateTime<Utc> = r
                    .try_get("published_at")
                    .map_err(|e| ApiError::Database(e.to_string()))?;
                let observed_slot: Option<i64> = r
                    .try_get("observed_slot")
                    .map_err(|e| ApiError::Database(e.to_string()))?;

                let p = Price::new(
                    price_mantissa,
                    exponent,
                    confidence_mantissa as u64,
                    published_at.timestamp(),
                    observed_slot.unwrap_or(0) as u64,
                );
                return Ok(Some(p));
            }
        }

        Ok(None)
    }
}

/// Repository for persisting and retrieving recovery recommendations.
#[derive(Debug, Clone)]
pub struct RecommendationRepository {
    pool: DatabasePool,
    memory_cache: Arc<RwLock<HashMap<Uuid, RecoveryRecommendationRecord>>>,
}

impl RecommendationRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self {
            pool,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn save_recommendation(
        &self,
        recovery_id: Uuid,
        position_id: &str,
        request_params: Value,
        ranked_response: Value,
        status: &str,
        ttl_secs: i64,
    ) -> Result<(), ApiError> {
        let now = Utc::now();
        let expires_at = now + Duration::seconds(ttl_secs);

        let record = RecoveryRecommendationRecord {
            recovery_id,
            position_id: position_id.to_string(),
            request_params: request_params.clone(),
            ranked_response: ranked_response.clone(),
            status: status.to_string(),
            created_at: now,
            expires_at,
        };

        self.memory_cache
            .write()
            .await
            .insert(recovery_id, record.clone());

        if let Some(pg) = self.pool.pool() {
            let _ = sqlx::query(
                r#"
                INSERT INTO recovery_recommendations (
                    recovery_id, position_id, request_params, ranked_response,
                    status, created_at, expires_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (recovery_id) DO UPDATE SET
                    status = EXCLUDED.status,
                    ranked_response = EXCLUDED.ranked_response
                "#,
            )
            .bind(recovery_id)
            .bind(position_id)
            .bind(request_params)
            .bind(ranked_response)
            .bind(status)
            .bind(now)
            .bind(expires_at)
            .execute(pg)
            .await;
        }

        Ok(())
    }

    pub async fn get_recommendation(
        &self,
        recovery_id: Uuid,
    ) -> Result<Option<RecoveryRecommendationRecord>, ApiError> {
        if let Some(rec) = self.memory_cache.read().await.get(&recovery_id).cloned() {
            return Ok(Some(rec));
        }

        if let Some(pg) = self.pool.pool() {
            let row = sqlx::query_as::<_, RecoveryRecommendationRecord>(
                r#"
                SELECT recovery_id, position_id, request_params, ranked_response,
                       status, created_at, expires_at
                FROM recovery_recommendations
                WHERE recovery_id = $1
                LIMIT 1
                "#,
            )
            .bind(recovery_id)
            .fetch_optional(pg)
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?;

            return Ok(row);
        }

        Ok(None)
    }
}
