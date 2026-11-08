use std::sync::Arc;

use domain_types::MarketConfig;

use crate::error::ApiError;
use crate::providers::MarketProvider;

#[derive(Clone)]
pub struct MarketService {
    provider: Arc<dyn MarketProvider>,
}

impl MarketService {
    pub fn new(provider: Arc<dyn MarketProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_market(&self, market_id: u64) -> Result<MarketConfig, ApiError> {
        self.provider
            .get_market(market_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Market {} not found", market_id)))
    }

    pub async fn list_markets(&self) -> Result<Vec<MarketConfig>, ApiError> {
        self.provider.list_markets().await
    }
}
