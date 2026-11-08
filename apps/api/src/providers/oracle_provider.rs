use std::future::Future;
use std::pin::Pin;

use domain_types::{AssetId, Price};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Provenance metadata for oracle price observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OracleSource {
    MainnetObserved,
    DevnetExecuted,
    SurfpoolSimulated,
    SyntheticFixture,
}

impl OracleSource {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::MainnetObserved => "Mainnet Observed",
            Self::DevnetExecuted => "Devnet Executed",
            Self::SurfpoolSimulated => "Surfpool Simulated",
            Self::SyntheticFixture => "Synthetic / Test Fixture",
        }
    }
}

/// Normalized oracle observation with price, provenance, validity, and publication metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleObservation {
    pub price: Price,
    pub feed_id: Option<String>,
    pub source: OracleSource,
    pub is_valid: bool,
    pub age_secs: i64,
    pub slot: Option<u64>,
}

pub trait OracleProvider: Send + Sync {
    fn get_price<'a>(
        &'a self,
        asset_id: &'a AssetId,
    ) -> BoxFuture<'a, Result<Option<Price>, ApiError>>;

    fn get_observation<'a>(
        &'a self,
        asset_id: &'a AssetId,
    ) -> BoxFuture<'a, Result<Option<OracleObservation>, ApiError>> {
        Box::pin(async move {
            let price_opt = self.get_price(asset_id).await?;
            Ok(price_opt.map(|price| OracleObservation {
                price,
                feed_id: None,
                source: OracleSource::SyntheticFixture,
                is_valid: true,
                age_secs: 0,
                slot: None,
            }))
        })
    }
}
