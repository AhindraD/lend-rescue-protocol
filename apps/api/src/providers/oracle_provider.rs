use std::future::Future;
use std::pin::Pin;

use domain_types::{AssetId, Price};

use crate::error::ApiError;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait OracleProvider: Send + Sync {
    fn get_price<'a>(
        &'a self,
        asset_id: &'a AssetId,
    ) -> BoxFuture<'a, Result<Option<Price>, ApiError>>;
}
