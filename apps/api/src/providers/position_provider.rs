use std::future::Future;
use std::pin::Pin;

use domain_types::PositionSnapshot;

use crate::error::ApiError;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait PositionProvider: Send + Sync {
    fn get_position<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<Option<PositionSnapshot>, ApiError>>;

    fn list_positions<'a>(
        &'a self,
        owner: Option<&'a str>,
        protocol: Option<&'a str>,
        network: Option<&'a str>,
    ) -> BoxFuture<'a, Result<Vec<PositionSnapshot>, ApiError>>;
}
