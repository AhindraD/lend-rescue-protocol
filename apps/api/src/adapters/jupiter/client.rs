use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;

use super::config::JupiterConfig;
use super::error::JupiterError;
use super::models::JupiterOrderResponseJson;
use crate::providers::position_provider::BoxFuture;

/// Abstract transport trait for interacting with Jupiter Swap API V2.
pub trait JupiterTransport: Send + Sync {
    fn get_order<'a>(
        &'a self,
        input_mint: &'a str,
        output_mint: &'a str,
        amount: u64,
        slippage_bps: Option<u16>,
    ) -> BoxFuture<'a, Result<JupiterOrderResponseJson, JupiterError>>;
}

/// Production HTTP transport querying Jupiter Swap API V2 `/order` in strictly quote-only mode.
#[derive(Clone)]
pub struct JupiterHttpTransport {
    config: JupiterConfig,
    client: reqwest::Client,
}

impl JupiterHttpTransport {
    pub fn new(config: JupiterConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.request_timeout_ms))
            .connect_timeout(Duration::from_millis(config.request_timeout_ms / 2))
            .pool_idle_timeout(Duration::from_secs(90))
            .user_agent("LendRescue/0.1.0 (Recovery Engine)")
            .build()
            .unwrap_or_default();

        Self { config, client }
    }
}

impl JupiterTransport for JupiterHttpTransport {
    fn get_order<'a>(
        &'a self,
        input_mint: &'a str,
        output_mint: &'a str,
        amount: u64,
        slippage_bps: Option<u16>,
    ) -> BoxFuture<'a, Result<JupiterOrderResponseJson, JupiterError>> {
        Box::pin(async move {
            // Validate inputs
            if input_mint.is_empty() || output_mint.is_empty() {
                return Err(JupiterError::InvalidMint(
                    "Mint address cannot be empty".to_string(),
                ));
            }
            if input_mint == output_mint {
                return Err(JupiterError::InvalidMint(
                    "Input and output mints cannot be identical".to_string(),
                ));
            }
            if amount == 0 {
                return Err(JupiterError::InvalidAmount(
                    "Swap amount must be greater than zero".to_string(),
                ));
            }

            // Construct strictly quote-only URL without taker parameter
            let mut url = format!(
                "{}/order?inputMint={}&outputMint={}&amount={}",
                self.config.api_url.trim_end_matches('/'),
                input_mint,
                output_mint,
                amount
            );

            if let Some(bps) = slippage_bps {
                url.push_str(&format!("&slippageBps={}", bps));
            }

            let mut retries = 0;
            let max_retries = self.config.max_retries;

            loop {
                let mut req = self.client.get(&url);
                if let Some(ref key) = self.config.api_key {
                    req = req.header("x-api-key", key);
                }

                match req.send().await {
                    Ok(resp) => {
                        let status = resp.status();

                        if status.is_success() {
                            let text = resp.text().await.map_err(|e| {
                                JupiterError::Http(format!("Failed to read response body: {}", e))
                            })?;

                            let parsed: JupiterOrderResponseJson = serde_json::from_str(&text)
                                .map_err(|e| {
                                    JupiterError::Serialization(format!(
                                        "JSON parsing error: {}",
                                        e
                                    ))
                                })?;

                            // Quote-only security assertion
                            if let Some(ref tx) = parsed.transaction {
                                if !tx.is_null() {
                                    return Err(JupiterError::UnexpectedTransactionPayload);
                                }
                            }

                            return Ok(parsed);
                        }

                        // Rate limit handling (HTTP 429)
                        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                            let retry_after = resp
                                .headers()
                                .get("retry-after")
                                .and_then(|v| v.to_str().ok())
                                .and_then(|s| s.parse::<u64>().ok());
                            return Err(JupiterError::RateLimited { retry_after });
                        }

                        // Authentication failure
                        if status == reqwest::StatusCode::UNAUTHORIZED
                            || status == reqwest::StatusCode::FORBIDDEN
                        {
                            return Err(JupiterError::AuthenticationFailed(format!(
                                "Jupiter rejected API credentials with HTTP {}",
                                status
                            )));
                        }

                        // Bad request / Not found / No route
                        if status == reqwest::StatusCode::BAD_REQUEST
                            || status == reqwest::StatusCode::NOT_FOUND
                        {
                            let body = resp.text().await.unwrap_or_default();
                            return Err(JupiterError::NoRouteFound(format!(
                                "Jupiter HTTP {}: {}",
                                status, body
                            )));
                        }

                        // Server errors (5xx) with retry
                        if status.is_server_error() && retries < max_retries {
                            retries += 1;
                            tokio::time::sleep(Duration::from_millis(100 * (1 << retries))).await;
                            continue;
                        }

                        return Err(JupiterError::Http(format!(
                            "HTTP error {}: {}",
                            status,
                            resp.text().await.unwrap_or_default()
                        )));
                    }
                    Err(e) => {
                        if e.is_timeout() {
                            return Err(JupiterError::Timeout(format!("Request timed out: {}", e)));
                        }
                        if retries < max_retries {
                            retries += 1;
                            tokio::time::sleep(Duration::from_millis(100 * (1 << retries))).await;
                            continue;
                        }
                        return Err(JupiterError::Http(format!("Connection error: {}", e)));
                    }
                }
            }
        })
    }
}

/// Recorded request for assertion testing.
#[derive(Debug, Clone)]
pub struct RecordedJupiterRequest {
    pub url: String,
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub slippage_bps: Option<u16>,
    pub has_api_key: bool,
    pub has_taker: bool,
}

/// Deterministic mock transport for offline unit and integration testing.
pub struct MockJupiterTransport {
    fixtures: RwLock<HashMap<(String, String, u64), JupiterOrderResponseJson>>,
    requests: Arc<RwLock<Vec<RecordedJupiterRequest>>>,
    force_rate_limit: RwLock<Option<u64>>,
    force_timeout: AtomicBool,
    force_auth_failure: AtomicBool,
    force_bad_request: RwLock<Option<String>>,
    force_server_error: AtomicBool,
    force_unexpected_transaction: AtomicBool,
    force_no_route: AtomicBool,
    expected_api_key: Option<String>,
}

impl Default for MockJupiterTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl MockJupiterTransport {
    pub fn new() -> Self {
        let mut fixtures = HashMap::new();

        // Standard Golden Fixture 1: 1 SOL -> 150 USDC (1_000_000_000 raw SOL -> 150_000_000 raw USDC)
        let sol_mint = "So11111111111111111111111111111111111111112";
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        fixtures.insert(
            (sol_mint.to_string(), usdc_mint.to_string(), 1_000_000_000),
            JupiterOrderResponseJson {
                in_amount: "1000000000".to_string(),
                out_amount: "150000000".to_string(),
                fee_bps: 10,
                fee_mint: Some(usdc_mint.to_string()),
                platform_fee: Some(super::models::JupiterPlatformFeeJson {
                    amount: Some("0".to_string()),
                    fee_bps: 0,
                }),
                price_impact_pct: Some("0.02".to_string()),
                route_plan: vec![super::models::JupiterRoutePlanStepJson {
                    swap_info: super::models::JupiterSwapInfoJson {
                        amm_key: Some("RaydiumCP1111111111111111111111111111111111".to_string()),
                        label: Some("Raydium CP".to_string()),
                        input_mint: sol_mint.to_string(),
                        output_mint: usdc_mint.to_string(),
                        in_amount: "1000000000".to_string(),
                        out_amount: "150000000".to_string(),
                        fee_amount: Some("150000".to_string()),
                        fee_mint: Some(usdc_mint.to_string()),
                    },
                    percent: 100,
                }],
                router: Some("iris".to_string()),
                mode: Some("ExactIn".to_string()),
                transaction: None,
            },
        );

        // Standard Golden Fixture 2: 1 bSOL -> 165 USDC
        let bsol_mint = "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1";
        fixtures.insert(
            (bsol_mint.to_string(), usdc_mint.to_string(), 1_000_000_000),
            JupiterOrderResponseJson {
                in_amount: "1000000000".to_string(),
                out_amount: "165000000".to_string(),
                fee_bps: 12,
                fee_mint: Some(usdc_mint.to_string()),
                platform_fee: None,
                price_impact_pct: Some("0.01".to_string()),
                route_plan: vec![super::models::JupiterRoutePlanStepJson {
                    swap_info: super::models::JupiterSwapInfoJson {
                        amm_key: Some("OrcaWhirlpool111111111111111111111111111111".to_string()),
                        label: Some("Orca Whirlpool".to_string()),
                        input_mint: bsol_mint.to_string(),
                        output_mint: usdc_mint.to_string(),
                        in_amount: "1000000000".to_string(),
                        out_amount: "165000000".to_string(),
                        fee_amount: Some("198000".to_string()),
                        fee_mint: Some(usdc_mint.to_string()),
                    },
                    percent: 100,
                }],
                router: Some("iris".to_string()),
                mode: Some("ExactIn".to_string()),
                transaction: None,
            },
        );

        Self {
            fixtures: RwLock::new(fixtures),
            requests: Arc::new(RwLock::new(Vec::new())),
            force_rate_limit: RwLock::new(None),
            force_timeout: AtomicBool::new(false),
            force_auth_failure: AtomicBool::new(false),
            force_bad_request: RwLock::new(None),
            force_server_error: AtomicBool::new(false),
            force_unexpected_transaction: AtomicBool::new(false),
            force_no_route: AtomicBool::new(false),
            expected_api_key: None,
        }
    }

    pub fn with_expected_api_key(mut self, key: Option<String>) -> Self {
        self.expected_api_key = key;
        self
    }

    pub async fn add_fixture(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        response: JupiterOrderResponseJson,
    ) {
        let mut fixtures = self.fixtures.write().await;
        fixtures.insert(
            (input_mint.to_string(), output_mint.to_string(), amount),
            response,
        );
    }

    pub async fn set_force_rate_limit(&self, retry_after: Option<u64>) {
        *self.force_rate_limit.write().await = retry_after;
    }

    pub fn set_force_timeout(&self, timeout: bool) {
        self.force_timeout.store(timeout, Ordering::SeqCst);
    }

    pub fn set_force_auth_failure(&self, auth_fail: bool) {
        self.force_auth_failure.store(auth_fail, Ordering::SeqCst);
    }

    pub async fn set_force_bad_request(&self, err_msg: Option<String>) {
        *self.force_bad_request.write().await = err_msg;
    }

    pub fn set_force_server_error(&self, server_err: bool) {
        self.force_server_error.store(server_err, Ordering::SeqCst);
    }

    pub fn set_force_unexpected_transaction(&self, unexpected: bool) {
        self.force_unexpected_transaction
            .store(unexpected, Ordering::SeqCst);
    }

    pub fn set_force_no_route(&self, no_route: bool) {
        self.force_no_route.store(no_route, Ordering::SeqCst);
    }

    pub async fn get_requests(&self) -> Vec<RecordedJupiterRequest> {
        self.requests.read().await.clone()
    }
}

impl JupiterTransport for MockJupiterTransport {
    fn get_order<'a>(
        &'a self,
        input_mint: &'a str,
        output_mint: &'a str,
        amount: u64,
        slippage_bps: Option<u16>,
    ) -> BoxFuture<'a, Result<JupiterOrderResponseJson, JupiterError>> {
        Box::pin(async move {
            let url = format!(
                "https://api.jup.ag/swap/v2/order?inputMint={}&outputMint={}&amount={}",
                input_mint, output_mint, amount
            );

            // Record request for test assertions
            {
                let mut reqs = self.requests.write().await;
                reqs.push(RecordedJupiterRequest {
                    url,
                    input_mint: input_mint.to_string(),
                    output_mint: output_mint.to_string(),
                    amount,
                    slippage_bps,
                    has_api_key: self.expected_api_key.is_some(),
                    has_taker: false, // Invariant: taker is NEVER passed!
                });
            }

            // Check simulated error conditions
            if let Some(secs) = *self.force_rate_limit.read().await {
                return Err(JupiterError::RateLimited {
                    retry_after: Some(secs),
                });
            }

            if self.force_timeout.load(Ordering::SeqCst) {
                return Err(JupiterError::Timeout("Mock timeout simulated".to_string()));
            }

            if self.force_auth_failure.load(Ordering::SeqCst) {
                return Err(JupiterError::AuthenticationFailed(
                    "Invalid API key provided".to_string(),
                ));
            }

            if let Some(ref msg) = *self.force_bad_request.read().await {
                return Err(JupiterError::InvalidAmount(msg.clone()));
            }

            if self.force_server_error.load(Ordering::SeqCst) {
                return Err(JupiterError::Http(
                    "HTTP 500 Internal Server Error".to_string(),
                ));
            }

            if self.force_no_route.load(Ordering::SeqCst) {
                return Err(JupiterError::NoRouteFound(format!(
                    "No route between {} and {}",
                    input_mint, output_mint
                )));
            }

            if self.force_unexpected_transaction.load(Ordering::SeqCst) {
                return Ok(JupiterOrderResponseJson {
                    in_amount: amount.to_string(),
                    out_amount: "150000000".to_string(),
                    fee_bps: 10,
                    fee_mint: Some(output_mint.to_string()),
                    platform_fee: None,
                    price_impact_pct: Some("0.01".to_string()),
                    route_plan: vec![],
                    router: Some("iris".to_string()),
                    mode: Some("ExactIn".to_string()),
                    transaction: Some(serde_json::Value::String(
                        "BASE64_TX_UNEXPECTED".to_string(),
                    )),
                });
            }

            // Look up fixture
            let fixtures = self.fixtures.read().await;
            if let Some(resp) =
                fixtures.get(&(input_mint.to_string(), output_mint.to_string(), amount))
            {
                return Ok(resp.clone());
            }

            // Fallback for custom amounts: compute proportional output if a base fixture exists
            for ((in_m, out_m, base_in), base_resp) in fixtures.iter() {
                if in_m == input_mint && out_m == output_mint && *base_in > 0 {
                    let base_out = base_resp.out_amount.parse::<u64>().unwrap_or(0);
                    let scaled_out =
                        ((amount as u128) * (base_out as u128) / (*base_in as u128)) as u64;

                    let mut custom_resp = base_resp.clone();
                    custom_resp.in_amount = amount.to_string();
                    custom_resp.out_amount = scaled_out.to_string();
                    return Ok(custom_resp);
                }
            }

            Err(JupiterError::NoRouteFound(format!(
                "No mock quote found for {} -> {} with amount {}",
                input_mint, output_mint, amount
            )))
        })
    }
}
