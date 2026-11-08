use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::error::PythError;
use crate::providers::oracle_provider::BoxFuture;

/// Raw account data fetched from Solana RPC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountDataResult {
    pub owner: String,
    pub data: Vec<u8>,
    pub slot: u64,
}

/// Abstract transport trait for fetching account data from Solana RPC.
pub trait SolanaRpcTransport: Send + Sync {
    fn get_account_data<'a>(
        &'a self,
        pubkey: &'a str,
    ) -> BoxFuture<'a, Result<Option<AccountDataResult>, PythError>>;
}

/// Hermes price update payload from Hermes HTTP API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermesPriceUpdate {
    pub feed_id: String,
    pub price: i64,
    pub conf: u64,
    pub expo: i32,
    pub publish_time: i64,
    pub slot: Option<u64>,
}

/// Abstract transport trait for fetching updates from Pyth Hermes API.
pub trait HermesTransport: Send + Sync {
    fn get_latest_price_update<'a>(
        &'a self,
        feed_id_hex: &'a str,
    ) -> BoxFuture<'a, Result<HermesPriceUpdate, PythError>>;
}

/// Mock in-memory Solana RPC transport for offline testing and deterministic failure injection.
#[derive(Default, Clone)]
pub struct MockSolanaRpcTransport {
    accounts: Arc<RwLock<HashMap<String, AccountDataResult>>>,
    should_timeout: Arc<AtomicBool>,
    simulated_error: Arc<RwLock<Option<String>>>,
}

impl MockSolanaRpcTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn set_account(&self, pubkey: impl Into<String>, account: AccountDataResult) {
        let mut map = self.accounts.write().await;
        map.insert(pubkey.into(), account);
    }

    pub fn set_timeout(&self, timeout: bool) {
        self.should_timeout.store(timeout, Ordering::SeqCst);
    }

    pub async fn set_error(&self, error: Option<String>) {
        let mut err = self.simulated_error.write().await;
        *err = error;
    }
}

impl SolanaRpcTransport for MockSolanaRpcTransport {
    fn get_account_data<'a>(
        &'a self,
        pubkey: &'a str,
    ) -> BoxFuture<'a, Result<Option<AccountDataResult>, PythError>> {
        Box::pin(async move {
            if self.should_timeout.load(Ordering::SeqCst) {
                return Err(PythError::Timeout(5000));
            }

            if let Some(err_msg) = self.simulated_error.read().await.as_ref() {
                return Err(PythError::RpcTransport(err_msg.clone()));
            }

            let map = self.accounts.read().await;
            Ok(map.get(pubkey).cloned())
        })
    }
}

/// Mock in-memory Hermes transport for offline testing.
#[derive(Default, Clone)]
pub struct MockHermesTransport {
    updates: Arc<RwLock<HashMap<String, HermesPriceUpdate>>>,
    require_auth: Arc<AtomicBool>,
    valid_api_key: Arc<RwLock<Option<String>>>,
    caller_api_key: Arc<RwLock<Option<String>>>,
    should_timeout: Arc<AtomicBool>,
}

impl MockHermesTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn set_update(&self, feed_id_hex: impl Into<String>, update: HermesPriceUpdate) {
        let mut map = self.updates.write().await;
        map.insert(feed_id_hex.into().to_lowercase(), update);
    }

    pub fn set_require_auth(&self, require: bool) {
        self.require_auth.store(require, Ordering::SeqCst);
    }

    pub async fn set_valid_api_key(&self, key: Option<String>) {
        let mut k = self.valid_api_key.write().await;
        *k = key;
    }

    pub async fn set_caller_api_key(&self, key: Option<String>) {
        let mut k = self.caller_api_key.write().await;
        *k = key;
    }

    pub fn set_timeout(&self, timeout: bool) {
        self.should_timeout.store(timeout, Ordering::SeqCst);
    }
}

impl HermesTransport for MockHermesTransport {
    fn get_latest_price_update<'a>(
        &'a self,
        feed_id_hex: &'a str,
    ) -> BoxFuture<'a, Result<HermesPriceUpdate, PythError>> {
        Box::pin(async move {
            if self.should_timeout.load(Ordering::SeqCst) {
                return Err(PythError::Timeout(5000));
            }

            if self.require_auth.load(Ordering::SeqCst) {
                let valid = self.valid_api_key.read().await;
                let caller = self.caller_api_key.read().await;
                match (valid.as_ref(), caller.as_ref()) {
                    (Some(expected), Some(actual)) if expected == actual => {}
                    _ => {
                        return Err(PythError::HermesUnauthorized(
                            "Invalid or missing Pyth API key".to_string(),
                        ));
                    }
                }
            }

            let map = self.updates.read().await;
            map.get(&feed_id_hex.to_lowercase())
                .cloned()
                .ok_or_else(|| PythError::MissingFeedMapping(feed_id_hex.to_string()))
        })
    }
}

