use chrono::Utc;
use domain_types::{AssetId, Price};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::ApiError;
use crate::providers::oracle_provider::{
    BoxFuture, OracleObservation, OracleProvider, OracleSource,
};

/// Synthetic Mock Oracle provider supporting deterministic test fixtures and price shock simulation.
#[derive(Clone)]
pub struct MockOracleProvider {
    prices: Arc<RwLock<HashMap<AssetId, Price>>>,
    source: OracleSource,
}

impl Default for MockOracleProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MockOracleProvider {
    /// Creates a mock oracle provider with standard default prices (SOL = $150, USDC = $1.00).
    pub fn new() -> Self {
        let now = Utc::now().timestamp();
        let mut prices = HashMap::new();

        prices.insert(
            AssetId::Sol,
            Price::new(150_00000000, -8, 1_00000000, now, 100).with_asset(AssetId::Sol),
        );
        prices.insert(
            AssetId::SSol,
            Price::new(150_00000000, -8, 1_00000000, now, 100).with_asset(AssetId::SSol),
        );
        prices.insert(
            AssetId::Usdc,
            Price::new(1_00000000, -8, 1000000, now, 100).with_asset(AssetId::Usdc),
        );
        prices.insert(
            AssetId::SUsdc,
            Price::new(1_00000000, -8, 1000000, now, 100).with_asset(AssetId::SUsdc),
        );

        Self {
            prices: Arc::new(RwLock::new(prices)),
            source: OracleSource::SyntheticFixture,
        }
    }

    /// Sets explicit source label (e.g. SyntheticFixture or DevnetExecuted).
    pub fn with_source(mut self, source: OracleSource) -> Self {
        self.source = source;
        self
    }

    /// Sets explicit price for an asset.
    pub async fn set_price(&self, asset_id: AssetId, price: Price) {
        let mut map = self.prices.write().await;
        map.insert(asset_id, price);
    }

    /// Simulates a price shock for testing recovery feasibility.
    pub async fn simulate_price_shock(&self, asset_id: AssetId, new_mantissa: i64) {
        let now = Utc::now().timestamp();
        let mut map = self.prices.write().await;
        if let Some(price) = map.get_mut(&asset_id) {
            price.mantissa = new_mantissa;
            price.timestamp = now;
        } else {
            map.insert(
                asset_id.clone(),
                Price::new(new_mantissa, -8, 1000000, now, 100).with_asset(asset_id),
            );
        }
    }
}

impl OracleProvider for MockOracleProvider {
    fn get_price<'a>(
        &'a self,
        asset_id: &'a AssetId,
    ) -> BoxFuture<'a, Result<Option<Price>, ApiError>> {
        Box::pin(async move {
            let map = self.prices.read().await;
            Ok(map.get(asset_id).cloned())
        })
    }

    fn get_observation<'a>(
        &'a self,
        asset_id: &'a AssetId,
    ) -> BoxFuture<'a, Result<Option<OracleObservation>, ApiError>> {
        Box::pin(async move {
            let map = self.prices.read().await;
            let price_opt = map.get(asset_id).cloned();
            let now = Utc::now().timestamp();

            Ok(price_opt.map(|price| {
                let age_secs = now - price.timestamp;
                OracleObservation {
                    price: price.clone(),
                    feed_id: None,
                    source: self.source,
                    is_valid: true,
                    age_secs,
                    slot: Some(price.slot),
                }
            }))
        })
    }
}
