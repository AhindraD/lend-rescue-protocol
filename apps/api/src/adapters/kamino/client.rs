use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use domain_types::{
    KaminoBorrowSnapshot, KaminoDepositSnapshot, KaminoMarketSnapshot, KaminoObligationSnapshot,
    KaminoProtocolMetrics, KaminoReserveSnapshot, ProtocolSource, TokenAmount,
};
use tokio::sync::RwLock;

use super::config::KaminoAdapterConfig;
use super::models::{
    KaminoMarketResponseJson, KaminoObligationResponseJson, KaminoObligationsListResponseJson,
    KaminoReserveResponseJson,
};
use crate::error::ApiError;
use crate::providers::position_provider::BoxFuture;

/// Abstract transport trait for communicating with Kamino Adapter.
pub trait KaminoTransport: Send + Sync {
    fn get_health(&self) -> BoxFuture<'_, Result<serde_json::Value, ApiError>>;
    fn get_market<'a>(
        &'a self,
        market_id: &'a str,
    ) -> BoxFuture<'a, Result<Option<KaminoMarketSnapshot>, ApiError>>;
    fn get_reserves<'a>(
        &'a self,
        market_id: &'a str,
    ) -> BoxFuture<'a, Result<Vec<KaminoReserveSnapshot>, ApiError>>;
    fn get_obligations<'a>(
        &'a self,
        market_id: &'a str,
        owner: &'a str,
    ) -> BoxFuture<'a, Result<Vec<KaminoObligationSnapshot>, ApiError>>;
    fn get_obligation<'a>(
        &'a self,
        market_id: &'a str,
        owner: &'a str,
        obligation_id: &'a str,
    ) -> BoxFuture<'a, Result<Option<KaminoObligationSnapshot>, ApiError>>;
}

/// Production HTTP transport communicating with the local isolated TypeScript Kamino adapter.
#[derive(Clone)]
pub struct KaminoHttpTransport {
    config: KaminoAdapterConfig,
    client: reqwest::Client,
}

impl KaminoHttpTransport {
    pub fn new(config: KaminoAdapterConfig) -> Self {
        let timeout = Duration::from_millis(config.request_timeout_ms);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_default();

        Self { config, client }
    }
}

impl KaminoTransport for KaminoHttpTransport {
    fn get_health(&self) -> BoxFuture<'_, Result<serde_json::Value, ApiError>> {
        Box::pin(async move {
            let url = format!("{}/health", self.config.adapter_url.trim_end_matches('/'));
            let res = self.client.get(&url).send().await.map_err(|e| {
                ApiError::Internal(format!("Kamino adapter health check failed: {}", e))
            })?;

            if !res.status().is_success() {
                return Err(ApiError::Internal(format!(
                    "Kamino adapter returned status {}",
                    res.status()
                )));
            }

            res.json::<serde_json::Value>().await.map_err(|e| {
                ApiError::Internal(format!("Failed to parse adapter health JSON: {}", e))
            })
        })
    }

    fn get_market<'a>(
        &'a self,
        market_id: &'a str,
    ) -> BoxFuture<'a, Result<Option<KaminoMarketSnapshot>, ApiError>> {
        Box::pin(async move {
            let url = format!(
                "{}/markets/{}",
                self.config.adapter_url.trim_end_matches('/'),
                market_id
            );
            let res = self.client.get(&url).send().await.map_err(|e| {
                if e.is_timeout() {
                    ApiError::Timeout(format!("Kamino market request timed out: {}", e))
                } else {
                    ApiError::Internal(format!("Kamino market request failed: {}", e))
                }
            })?;

            if res.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }

            if !res.status().is_success() {
                return Err(ApiError::Internal(format!(
                    "Kamino market request failed with status {}",
                    res.status()
                )));
            }

            let json: KaminoMarketResponseJson = res.json().await.map_err(|e| {
                ApiError::Internal(format!("Failed to decode Kamino market response: {}", e))
            })?;

            Ok(Some(json.to_domain()))
        })
    }

    fn get_reserves<'a>(
        &'a self,
        market_id: &'a str,
    ) -> BoxFuture<'a, Result<Vec<KaminoReserveSnapshot>, ApiError>> {
        Box::pin(async move {
            let url = format!(
                "{}/reserves/{}",
                self.config.adapter_url.trim_end_matches('/'),
                market_id
            );
            let res = self.client.get(&url).send().await.map_err(|e| {
                if e.is_timeout() {
                    ApiError::Timeout(format!("Kamino reserves request timed out: {}", e))
                } else {
                    ApiError::Internal(format!("Kamino reserves request failed: {}", e))
                }
            })?;

            if !res.status().is_success() {
                return Err(ApiError::Internal(format!(
                    "Kamino reserves request failed with status {}",
                    res.status()
                )));
            }

            let json: Vec<KaminoReserveResponseJson> = res.json().await.map_err(|e| {
                ApiError::Internal(format!("Failed to decode Kamino reserves response: {}", e))
            })?;

            Ok(json.into_iter().map(|r| r.to_domain()).collect())
        })
    }

    fn get_obligations<'a>(
        &'a self,
        market_id: &'a str,
        owner: &'a str,
    ) -> BoxFuture<'a, Result<Vec<KaminoObligationSnapshot>, ApiError>> {
        Box::pin(async move {
            let url = format!(
                "{}/obligations/{}/{}",
                self.config.adapter_url.trim_end_matches('/'),
                market_id,
                owner
            );
            let res = self.client.get(&url).send().await.map_err(|e| {
                if e.is_timeout() {
                    ApiError::Timeout(format!("Kamino obligations request timed out: {}", e))
                } else {
                    ApiError::Internal(format!("Kamino obligations request failed: {}", e))
                }
            })?;

            if !res.status().is_success() {
                return Err(ApiError::Internal(format!(
                    "Kamino obligations request failed with status {}",
                    res.status()
                )));
            }

            let list_json: KaminoObligationsListResponseJson = res.json().await.map_err(|e| {
                ApiError::Internal(format!(
                    "Failed to decode Kamino obligations response: {}",
                    e
                ))
            })?;

            let mut list = Vec::new();
            for item in list_json.obligations {
                list.push(item.to_domain()?);
            }
            Ok(list)
        })
    }

    fn get_obligation<'a>(
        &'a self,
        market_id: &'a str,
        owner: &'a str,
        obligation_id: &'a str,
    ) -> BoxFuture<'a, Result<Option<KaminoObligationSnapshot>, ApiError>> {
        Box::pin(async move {
            let url = if owner.is_empty() {
                format!(
                    "{}/obligations/{}/by-id/{}",
                    self.config.adapter_url.trim_end_matches('/'),
                    market_id,
                    obligation_id
                )
            } else {
                format!(
                    "{}/obligations/{}/{}/{}",
                    self.config.adapter_url.trim_end_matches('/'),
                    market_id,
                    owner,
                    obligation_id
                )
            };
            let res = self.client.get(&url).send().await.map_err(|e| {
                if e.is_timeout() {
                    ApiError::Timeout(format!("Kamino obligation request timed out: {}", e))
                } else {
                    ApiError::Internal(format!("Kamino obligation request failed: {}", e))
                }
            })?;

            if res.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }

            if !res.status().is_success() {
                return Err(ApiError::Internal(format!(
                    "Kamino obligation request failed with status {}",
                    res.status()
                )));
            }

            let json: KaminoObligationResponseJson = res.json().await.map_err(|e| {
                ApiError::Internal(format!(
                    "Failed to decode Kamino obligation response: {}",
                    e
                ))
            })?;

            Ok(Some(json.to_domain()?))
        })
    }
}

/// Deterministic mock transport for offline unit and CI tests without running the Node process.
#[derive(Clone)]
pub struct MockKaminoTransport {
    markets: Arc<RwLock<HashMap<String, KaminoMarketSnapshot>>>,
    obligations: Arc<RwLock<HashMap<String, KaminoObligationSnapshot>>>,
    should_timeout: Arc<AtomicBool>,
    simulated_error: Arc<RwLock<Option<String>>>,
}

impl Default for MockKaminoTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl MockKaminoTransport {
    pub fn new() -> Self {
        let mut markets = HashMap::new();
        let mut obligations = HashMap::new();

        let market_id = "7u3HeHxatcAACvdFtHGryaZWwZxKhqWisdHJaJBYbugj".to_string();

        let sol_reserve = KaminoReserveSnapshot {
            reserve_id: "So11111111111111111111111111111111111111112Reserve".to_string(),
            liquidity_mint: "So11111111111111111111111111111111111111112".to_string(),
            collateral_mint: "cSo11111111111111111111111111111111111111112".to_string(),
            token_symbol: "SOL".to_string(),
            decimals: 9,
            loan_to_value_bps: 7500,
            liquidation_threshold_bps: 8500,
            liquidation_bonus_bps: 500,
            available_liquidity: TokenAmount::new(50_000_000_000_000, 9, None),
            total_deposits: TokenAmount::new(120_000_000_000_000, 9, None),
            total_borrows: TokenAmount::new(70_000_000_000_000, 9, None),
            borrow_interest_rate_bps: 850,
            supply_interest_rate_bps: 425,
            oracle_address: "PythFeedSOL1111111111111111111111111111111111".to_string(),
            last_update_slot: 280_000_000,
            status: "active".to_string(),
        };

        let usdc_reserve = KaminoReserveSnapshot {
            reserve_id: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1vReserve".to_string(),
            liquidity_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            collateral_mint: "cEPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            token_symbol: "USDC".to_string(),
            decimals: 6,
            loan_to_value_bps: 8000,
            liquidation_threshold_bps: 8500,
            liquidation_bonus_bps: 500,
            available_liquidity: TokenAmount::new(20_000_000_000_000, 6, None),
            total_deposits: TokenAmount::new(45_000_000_000_000, 6, None),
            total_borrows: TokenAmount::new(25_000_000_000_000, 6, None),
            borrow_interest_rate_bps: 920,
            supply_interest_rate_bps: 510,
            oracle_address: "PythFeedUSDC111111111111111111111111111111111".to_string(),
            last_update_slot: 280_000_000,
            status: "active".to_string(),
        };

        let market_snap = KaminoMarketSnapshot {
            market_id: market_id.clone(),
            owner: "KaminoMarketOwner111111111111111111111111111".to_string(),
            authority: "KaminoMarketAuth111111111111111111111111111".to_string(),
            program_id: "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD".to_string(),
            network: "surfpool".to_string(),
            observation_slot: 280_000_100,
            observed_at: "2026-11-08T10:00:00Z".to_string(),
            reserves: vec![sol_reserve, usdc_reserve],
        };
        markets.insert(market_id.clone(), market_snap);

        // 1. Healthy Obligation: 15 SOL ($2,250), 1,000 USDC debt -> HF ~ 1.91
        let obl_healthy = KaminoObligationSnapshot {
            obligation_id: "kamino-obl-healthy".to_string(),
            owner: "kamino_user_1111111111111111111111111111111".to_string(),
            market_id: market_id.clone(),
            observation_slot: 280_000_100,
            observed_at: "2026-11-08T10:00:00Z".to_string(),
            protocol_source: ProtocolSource::SurfpoolSimulated,
            deposits: vec![KaminoDepositSnapshot {
                reserve_address: "So11111111111111111111111111111111111111112Reserve".to_string(),
                mint_address: "So11111111111111111111111111111111111111112".to_string(),
                token_symbol: "SOL".to_string(),
                decimals: 9,
                raw_amount: TokenAmount::new(15_000_000_000, 9, None),
                market_value_usd_scaled: 2_250_000_000, // $2,250
            }],
            borrows: vec![KaminoBorrowSnapshot {
                reserve_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1vReserve".to_string(),
                mint_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                token_symbol: "USDC".to_string(),
                decimals: 6,
                raw_amount: TokenAmount::new(1_000_000_000, 6, None),
                market_value_usd_scaled: 1_000_000_000, // $1,000
            }],
            protocol_metrics: KaminoProtocolMetrics {
                borrow_limit_usd_scaled: 1_687_500_000,          // 75% LTV
                liquidation_threshold_usd_scaled: 1_912_500_000, // 85% LT
                total_deposit_usd_scaled: 2_250_000_000,
                total_borrow_usd_scaled: 1_000_000_000,
                loan_to_value_bps: 4444,
                liquidation_threshold_bps: 8500,
                borrow_utilization_bps: 5925,
                net_account_value_usd_scaled: 1_250_000_000,
                is_protocol_reported: true,
            },
        };
        obligations.insert(obl_healthy.obligation_id.clone(), obl_healthy);

        // 2. Critical Obligation: 10 SOL ($1,500), 1,200 USDC debt -> HF = 1.0625
        let obl_critical = KaminoObligationSnapshot {
            obligation_id: "kamino-obl-critical".to_string(),
            owner: "kamino_user_2222222222222222222222222222222".to_string(),
            market_id: market_id.clone(),
            observation_slot: 280_000_100,
            observed_at: "2026-11-08T10:00:00Z".to_string(),
            protocol_source: ProtocolSource::SurfpoolSimulated,
            deposits: vec![KaminoDepositSnapshot {
                reserve_address: "So11111111111111111111111111111111111111112Reserve".to_string(),
                mint_address: "So11111111111111111111111111111111111111112".to_string(),
                token_symbol: "SOL".to_string(),
                decimals: 9,
                raw_amount: TokenAmount::new(10_000_000_000, 9, None),
                market_value_usd_scaled: 1_500_000_000,
            }],
            borrows: vec![KaminoBorrowSnapshot {
                reserve_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1vReserve".to_string(),
                mint_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                token_symbol: "USDC".to_string(),
                decimals: 6,
                raw_amount: TokenAmount::new(1_200_000_000, 6, None),
                market_value_usd_scaled: 1_200_000_000,
            }],
            protocol_metrics: KaminoProtocolMetrics {
                borrow_limit_usd_scaled: 1_125_000_000,
                liquidation_threshold_usd_scaled: 1_275_000_000,
                total_deposit_usd_scaled: 1_500_000_000,
                total_borrow_usd_scaled: 1_200_000_000,
                loan_to_value_bps: 8000,
                liquidation_threshold_bps: 8500,
                borrow_utilization_bps: 10666, // over borrow limit
                net_account_value_usd_scaled: 300_000_000,
                is_protocol_reported: true,
            },
        };
        obligations.insert(obl_critical.obligation_id.clone(), obl_critical);

        // 3. Multi-Asset Obligation: Deposits of SOL + bSOL, Borrows of USDC + USDT
        let obl_multi = KaminoObligationSnapshot {
            obligation_id: "kamino-obl-multi-asset".to_string(),
            owner: "kamino_user_multi1111111111111111111111111".to_string(),
            market_id: market_id.clone(),
            observation_slot: 280_000_100,
            observed_at: "2026-11-08T10:00:00Z".to_string(),
            protocol_source: ProtocolSource::SurfpoolSimulated,
            deposits: vec![
                KaminoDepositSnapshot {
                    reserve_address: "ReserveSOL".to_string(),
                    mint_address: "So11111111111111111111111111111111111111112".to_string(),
                    token_symbol: "SOL".to_string(),
                    decimals: 9,
                    raw_amount: TokenAmount::new(10_000_000_000, 9, None),
                    market_value_usd_scaled: 1_500_000_000,
                },
                KaminoDepositSnapshot {
                    reserve_address: "ReserveBSOL".to_string(),
                    mint_address: "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1".to_string(),
                    token_symbol: "bSOL".to_string(),
                    decimals: 9,
                    raw_amount: TokenAmount::new(5_000_000_000, 9, None),
                    market_value_usd_scaled: 850_000_000,
                },
            ],
            borrows: vec![
                KaminoBorrowSnapshot {
                    reserve_address: "ReserveUSDC".to_string(),
                    mint_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                    token_symbol: "USDC".to_string(),
                    decimals: 6,
                    raw_amount: TokenAmount::new(1_000_000_000, 6, None),
                    market_value_usd_scaled: 1_000_000_000,
                },
                KaminoBorrowSnapshot {
                    reserve_address: "ReserveUSDT".to_string(),
                    mint_address: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
                    token_symbol: "USDT".to_string(),
                    decimals: 6,
                    raw_amount: TokenAmount::new(500_000_000, 6, None),
                    market_value_usd_scaled: 500_000_000,
                },
            ],
            protocol_metrics: KaminoProtocolMetrics {
                borrow_limit_usd_scaled: 1_762_500_000,
                liquidation_threshold_usd_scaled: 1_997_500_000,
                total_deposit_usd_scaled: 2_350_000_000,
                total_borrow_usd_scaled: 1_500_000_000,
                loan_to_value_bps: 6382,
                liquidation_threshold_bps: 8500,
                borrow_utilization_bps: 8510,
                net_account_value_usd_scaled: 850_000_000,
                is_protocol_reported: true,
            },
        };
        obligations.insert(obl_multi.obligation_id.clone(), obl_multi);

        Self {
            markets: Arc::new(RwLock::new(markets)),
            obligations: Arc::new(RwLock::new(obligations)),
            should_timeout: Arc::new(AtomicBool::new(false)),
            simulated_error: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_timeout(&self, timeout: bool) {
        self.should_timeout.store(timeout, Ordering::SeqCst);
    }

    pub async fn set_error(&self, error: Option<String>) {
        let mut err = self.simulated_error.write().await;
        *err = error;
    }

    pub async fn add_obligation(&self, obl: KaminoObligationSnapshot) {
        let mut map = self.obligations.write().await;
        map.insert(obl.obligation_id.clone(), obl);
    }
}

impl KaminoTransport for MockKaminoTransport {
    fn get_health(&self) -> BoxFuture<'_, Result<serde_json::Value, ApiError>> {
        Box::pin(async move {
            if self.should_timeout.load(Ordering::SeqCst) {
                return Err(ApiError::Timeout("Mock timeout".to_string()));
            }
            if let Some(err) = self.simulated_error.read().await.as_ref() {
                return Err(ApiError::Internal(err.clone()));
            }

            Ok(serde_json::json!({
                "status": "ok",
                "version": "0.1.0",
                "sdkVersion": "13.0.6",
                "network": "surfpool",
                "readOnly": true,
            }))
        })
    }

    fn get_market<'a>(
        &'a self,
        market_id: &'a str,
    ) -> BoxFuture<'a, Result<Option<KaminoMarketSnapshot>, ApiError>> {
        Box::pin(async move {
            if self.should_timeout.load(Ordering::SeqCst) {
                return Err(ApiError::Timeout("Mock timeout".to_string()));
            }
            if let Some(err) = self.simulated_error.read().await.as_ref() {
                return Err(ApiError::Internal(err.clone()));
            }

            let map = self.markets.read().await;
            Ok(map.get(market_id).cloned())
        })
    }

    fn get_reserves<'a>(
        &'a self,
        market_id: &'a str,
    ) -> BoxFuture<'a, Result<Vec<KaminoReserveSnapshot>, ApiError>> {
        Box::pin(async move {
            if self.should_timeout.load(Ordering::SeqCst) {
                return Err(ApiError::Timeout("Mock timeout".to_string()));
            }
            if let Some(err) = self.simulated_error.read().await.as_ref() {
                return Err(ApiError::Internal(err.clone()));
            }

            let map = self.markets.read().await;
            Ok(map
                .get(market_id)
                .map(|m| m.reserves.clone())
                .unwrap_or_default())
        })
    }

    fn get_obligations<'a>(
        &'a self,
        market_id: &'a str,
        owner: &'a str,
    ) -> BoxFuture<'a, Result<Vec<KaminoObligationSnapshot>, ApiError>> {
        Box::pin(async move {
            if self.should_timeout.load(Ordering::SeqCst) {
                return Err(ApiError::Timeout("Mock timeout".to_string()));
            }
            if let Some(err) = self.simulated_error.read().await.as_ref() {
                return Err(ApiError::Internal(err.clone()));
            }

            let map = self.obligations.read().await;
            let list: Vec<KaminoObligationSnapshot> = map
                .values()
                .filter(|o| o.market_id == market_id && o.owner == owner)
                .cloned()
                .collect();
            Ok(list)
        })
    }

    fn get_obligation<'a>(
        &'a self,
        market_id: &'a str,
        owner: &'a str,
        obligation_id: &'a str,
    ) -> BoxFuture<'a, Result<Option<KaminoObligationSnapshot>, ApiError>> {
        Box::pin(async move {
            if self.should_timeout.load(Ordering::SeqCst) {
                return Err(ApiError::Timeout("Mock timeout".to_string()));
            }
            if let Some(err) = self.simulated_error.read().await.as_ref() {
                return Err(ApiError::Internal(err.clone()));
            }

            let map = self.obligations.read().await;
            if let Some(obl) = map.get(obligation_id) {
                if obl.market_id == market_id && (owner.is_empty() || obl.owner == owner) {
                    return Ok(Some(obl.clone()));
                }
            }
            Ok(None)
        })
    }
}
