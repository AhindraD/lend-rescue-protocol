use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde_json::json;
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

/// Live HTTP Solana JSON-RPC transport for reading real on-chain accounts (e.g. from Surfpool or Solana RPC).
pub struct HttpSolanaRpcTransport {
    rpc_url: String,
    client: reqwest::Client,
}

impl HttpSolanaRpcTransport {
    pub fn new(rpc_url: impl Into<String>, timeout_ms: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(timeout_ms))
            .build()
            .unwrap_or_default();
        Self {
            rpc_url: rpc_url.into(),
            client,
        }
    }
}

impl SolanaRpcTransport for HttpSolanaRpcTransport {
    fn get_account_data<'a>(
        &'a self,
        pubkey: &'a str,
    ) -> BoxFuture<'a, Result<Option<AccountDataResult>, PythError>> {
        Box::pin(async move {
            let body = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getAccountInfo",
                "params": [
                    pubkey,
                    {
                        "encoding": "base64",
                        "commitment": "confirmed"
                    }
                ]
            });

            let resp = self
                .client
                .post(&self.rpc_url)
                .json(&body)
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        PythError::Timeout(5000)
                    } else {
                        PythError::RpcTransport(format!("HTTP RPC request failed: {}", e))
                    }
                })?;

            if !resp.status().is_success() {
                return Err(PythError::RpcTransport(format!(
                    "HTTP RPC response error status: {}",
                    resp.status()
                )));
            }

            let resp_json: serde_json::Value = resp.json().await.map_err(|e| {
                PythError::RpcTransport(format!("Failed to parse JSON RPC response: {}", e))
            })?;

            if let Some(err) = resp_json.get("error") {
                return Err(PythError::RpcTransport(format!(
                    "Solana RPC returned error: {}",
                    err
                )));
            }

            let result = match resp_json.get("result") {
                Some(r) if !r.is_null() => r,
                _ => return Ok(None),
            };

            let val = match result.get("value") {
                Some(v) if !v.is_null() => v,
                _ => return Ok(None),
            };

            let owner = val
                .get("owner")
                .and_then(|o| o.as_str())
                .unwrap_or("")
                .to_string();

            let slot = result
                .get("context")
                .and_then(|c| c.get("slot"))
                .and_then(|s| s.as_u64())
                .unwrap_or(0);

            let data_b64 = val
                .get("data")
                .and_then(|d| d.get(0))
                .and_then(|s| s.as_str())
                .ok_or_else(|| {
                    PythError::InvalidAccountData(
                        "Missing base64 data array in getAccountInfo".to_string(),
                    )
                })?;

            let data = BASE64.decode(data_b64).map_err(|e| {
                PythError::InvalidAccountData(format!("Base64 decode failed: {}", e))
            })?;

            Ok(Some(AccountDataResult { owner, data, slot }))
        })
    }
}

/// Live HTTP Hermes transport for querying Pyth Hermes price update service.
pub struct HttpHermesTransport {
    hermes_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl HttpHermesTransport {
    pub fn new(hermes_url: impl Into<String>, api_key: Option<String>, timeout_ms: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(timeout_ms))
            .build()
            .unwrap_or_default();
        Self {
            hermes_url: hermes_url.into(),
            api_key,
            client,
        }
    }
}

impl HermesTransport for HttpHermesTransport {
    fn get_latest_price_update<'a>(
        &'a self,
        feed_id_hex: &'a str,
    ) -> BoxFuture<'a, Result<HermesPriceUpdate, PythError>> {
        Box::pin(async move {
            let url = format!(
                "{}/v2/updates/price/latest?ids[]={}",
                self.hermes_url.trim_end_matches('/'),
                feed_id_hex
            );

            let mut req = self.client.get(&url);
            if let Some(key) = &self.api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }

            let resp = req.send().await.map_err(|e| {
                if e.is_timeout() {
                    PythError::Timeout(5000)
                } else {
                    PythError::HermesError(format!("Hermes HTTP request failed: {}", e))
                }
            })?;

            if resp.status() == reqwest::StatusCode::UNAUTHORIZED
                || resp.status() == reqwest::StatusCode::FORBIDDEN
            {
                return Err(PythError::HermesUnauthorized(
                    "Invalid or missing Pyth API key for Hermes".to_string(),
                ));
            }

            if !resp.status().is_success() {
                return Err(PythError::HermesError(format!(
                    "Hermes returned error status: {}",
                    resp.status()
                )));
            }

            let resp_json: serde_json::Value = resp.json().await.map_err(|e| {
                PythError::HermesError(format!("Failed to parse Hermes response JSON: {}", e))
            })?;

            let parsed = resp_json
                .get("parsed")
                .and_then(|p| p.get(0))
                .ok_or_else(|| PythError::MissingFeedMapping(feed_id_hex.to_string()))?;

            let feed_id = parsed
                .get("id")
                .and_then(|id| id.as_str())
                .unwrap_or(feed_id_hex)
                .to_string();

            let price_data = parsed.get("price").ok_or_else(|| {
                PythError::InvalidAccountData("Missing price object in Hermes update".to_string())
            })?;

            let price: i64 = price_data
                .get("price")
                .and_then(|p| p.as_str())
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| PythError::InvalidAccountData("Invalid price value".to_string()))?;

            let conf: u64 = price_data
                .get("conf")
                .and_then(|c| c.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let expo: i32 = price_data
                .get("expo")
                .and_then(|e| e.as_i64())
                .map(|e| e as i32)
                .unwrap_or(-8);

            let publish_time: i64 = price_data
                .get("publish_time")
                .and_then(|t| t.as_i64())
                .unwrap_or(0);

            Ok(HermesPriceUpdate {
                feed_id,
                price,
                conf,
                expo,
                publish_time,
                slot: None,
            })
        })
    }
}
