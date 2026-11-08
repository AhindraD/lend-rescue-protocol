use std::future::Future;
use std::pin::Pin;

use domain_types::MarketConfig;

use crate::error::ApiError;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait MarketProvider: Send + Sync {
    fn get_market(&self, market_id: u64) -> BoxFuture<'_, Result<Option<MarketConfig>, ApiError>>;
    fn list_markets(&self) -> BoxFuture<'_, Result<Vec<MarketConfig>, ApiError>>;
}
