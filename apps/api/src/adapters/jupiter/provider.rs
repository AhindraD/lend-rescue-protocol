use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use domain_types::SwapQuote;
use tokio::sync::RwLock;

use super::client::JupiterTransport;
use super::config::JupiterConfig;
use crate::error::ApiError;
use crate::providers::position_provider::BoxFuture;

/// Validates that a string looks like a valid Solana base58 public key.
fn is_valid_base58_mint(s: &str) -> bool {
    if s.len() < 32 || s.len() > 44 {
        return false;
    }
    s.chars().all(|c| {
        matches!(c,
            '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z')
    })
}

pub type SwapQuoteCacheKey = (String, String, u64);
pub type SwapQuoteCache = Arc<RwLock<HashMap<SwapQuoteCacheKey, (SwapQuote, Instant)>>>;

/// Production quote provider for Jupiter Swap API V2.
#[derive(Clone)]
pub struct JupiterSwapQuoteProvider {
    config: JupiterConfig,
    transport: Arc<dyn JupiterTransport>,
    cache: SwapQuoteCache,
}

impl JupiterSwapQuoteProvider {
    pub fn new(config: JupiterConfig, transport: Arc<dyn JupiterTransport>) -> Self {
        Self {
            config,
            transport,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn config(&self) -> &JupiterConfig {
        &self.config
    }

    pub fn transport(&self) -> &Arc<dyn JupiterTransport> {
        &self.transport
    }

    /// Fetches an indicative quote for swapping `input_mint` to `output_mint` for `amount` base units.
    pub fn get_swap_quote<'a>(
        &'a self,
        input_mint: &'a str,
        output_mint: &'a str,
        amount: u64,
        max_slippage_bps: u16,
        refresh: bool,
    ) -> BoxFuture<'a, Result<SwapQuote, ApiError>> {
        Box::pin(async move {
            // 1. Validate mint formats
            if !is_valid_base58_mint(input_mint) {
                return Err(ApiError::BadRequest(format!(
                    "Invalid input token mint address '{}'",
                    input_mint
                )));
            }
            if !is_valid_base58_mint(output_mint) {
                return Err(ApiError::BadRequest(format!(
                    "Invalid output token mint address '{}'",
                    output_mint
                )));
            }
            if input_mint == output_mint {
                return Err(ApiError::BadRequest(
                    "Input and output mints cannot be identical".to_string(),
                ));
            }
            if amount == 0 {
                return Err(ApiError::BadRequest(
                    "Swap amount must be greater than zero".to_string(),
                ));
            }
            if max_slippage_bps > 10_000 {
                return Err(ApiError::BadRequest(
                    "Max slippage basis points cannot exceed 10,000".to_string(),
                ));
            }

            let cache_key = (input_mint.to_string(), output_mint.to_string(), amount);

            // 2. Check in-memory cache if not forcing refresh
            if !refresh {
                let cache = self.cache.read().await;
                if let Some((quote, cached_at)) = cache.get(&cache_key) {
                    let ttl = Duration::from_secs(self.config.quote_ttl_secs as u64);
                    if cached_at.elapsed() < ttl {
                        return Ok(quote.clone());
                    }
                }
            }

            // 3. Query transport
            let raw_resp = self
                .transport
                .get_order(input_mint, output_mint, amount, Some(max_slippage_bps))
                .await
                .map_err(ApiError::from)?;

            let now = Utc::now().timestamp();
            let quote = raw_resp
                .into_swap_quote(
                    input_mint,
                    output_mint,
                    amount,
                    max_slippage_bps,
                    now,
                    self.config.quote_ttl_secs,
                )
                .map_err(ApiError::from)?;

            // 4. Update cache
            {
                let mut cache = self.cache.write().await;
                cache.insert(cache_key, (quote.clone(), Instant::now()));
            }

            Ok(quote)
        })
    }
}
