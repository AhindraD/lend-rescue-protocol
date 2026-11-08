use std::collections::HashMap;
use std::sync::Arc;

use domain_types::{AssetId, MarketConfig, PositionSnapshot, Price, ProtocolSource, TokenAmount};
use recovery_engine::SwapPoolParams;

use crate::error::ApiError;
use crate::providers::market_provider::{BoxFuture, MarketProvider};
use crate::providers::oracle_provider::OracleProvider;
use crate::providers::position_provider::PositionProvider;

/// Deterministic in-memory provider supplying synthetic test fixtures.
#[derive(Debug, Clone)]
pub struct DemoProvider {
    markets: Arc<HashMap<u64, MarketConfig>>,
    positions: Arc<HashMap<String, PositionSnapshot>>,
    swap_pool: SwapPoolParams,
}

impl Default for DemoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DemoProvider {
    pub fn new() -> Self {
        let mut markets = HashMap::new();
        let mut positions = HashMap::new();

        // 1. Synthetic Demo Market ID 1
        let market1 = MarketConfig::new(
            1,
            "demo_market_authority",
            AssetId::SSol,
            AssetId::SUsdc,
            8500, // 85% liquidation threshold
            30,   // 30 bps swap fee
        );
        markets.insert(1, market1);

        // 2. Positions:
        // Position A: Healthy (15 sSOL, 1,000 sUSDC -> HF = 1.9125)
        positions.insert(
            "demo-pos-healthy".to_string(),
            PositionSnapshot {
                id: "demo-pos-healthy".to_string(),
                owner: "demo_user_11111111111111111111111111111111".to_string(),
                collateral_asset: AssetId::SSol,
                collateral_amount: TokenAmount::new(15_000_000_000, 9, Some(AssetId::SSol)),
                debt_asset: AssetId::SUsdc,
                debt_amount: TokenAmount::new(1_000_000_000, 6, Some(AssetId::SUsdc)),
                liquidation_threshold_bps: 8500,
                protocol_source: ProtocolSource::SyntheticDemo,
                market_id: Some(1),
            },
        );

        // Position B: Warning (10 sSOL, 1,100 sUSDC -> HF = 1.1590)
        positions.insert(
            "demo-pos-warning".to_string(),
            PositionSnapshot {
                id: "demo-pos-warning".to_string(),
                owner: "demo_user_11111111111111111111111111111111".to_string(),
                collateral_asset: AssetId::SSol,
                collateral_amount: TokenAmount::new(10_000_000_000, 9, Some(AssetId::SSol)),
                debt_asset: AssetId::SUsdc,
                debt_amount: TokenAmount::new(1_100_000_000, 6, Some(AssetId::SUsdc)),
                liquidation_threshold_bps: 8500,
                protocol_source: ProtocolSource::SyntheticDemo,
                market_id: Some(1),
            },
        );

        // Position C: Critical (Golden Scenario 1: 10 sSOL, 1,200 sUSDC -> HF = 1.0625)
        positions.insert(
            "demo-pos-critical".to_string(),
            PositionSnapshot {
                id: "demo-pos-critical".to_string(),
                owner: "demo_user_22222222222222222222222222222222".to_string(),
                collateral_asset: AssetId::SSol,
                collateral_amount: TokenAmount::new(10_000_000_000, 9, Some(AssetId::SSol)),
                debt_asset: AssetId::SUsdc,
                debt_amount: TokenAmount::new(1_200_000_000, 6, Some(AssetId::SUsdc)),
                liquidation_threshold_bps: 8500,
                protocol_source: ProtocolSource::SyntheticDemo,
                market_id: Some(1),
            },
        );

        // Position D: Liquidatable (10 sSOL, 1,350 sUSDC -> HF = 0.9444)
        positions.insert(
            "demo-pos-liquidatable".to_string(),
            PositionSnapshot {
                id: "demo-pos-liquidatable".to_string(),
                owner: "demo_user_33333333333333333333333333333333".to_string(),
                collateral_asset: AssetId::SSol,
                collateral_amount: TokenAmount::new(10_000_000_000, 9, Some(AssetId::SSol)),
                debt_asset: AssetId::SUsdc,
                debt_amount: TokenAmount::new(1_350_000_000, 6, Some(AssetId::SUsdc)),
                liquidation_threshold_bps: 8500,
                protocol_source: ProtocolSource::SyntheticDemo,
                market_id: Some(1),
            },
        );

        // Constant product demo swap pool: 1,000 SOL (9 decimals) and 150,000 USDC (6 decimals)
        let swap_pool = SwapPoolParams::new(1_000_000_000_000, 150_000_000_000, 30);

        Self {
            markets: Arc::new(markets),
            positions: Arc::new(positions),
            swap_pool,
        }
    }

    pub fn swap_pool_params(&self) -> SwapPoolParams {
        self.swap_pool
    }

    /// Fixed current synthetic timestamp for deterministic tests
    pub fn current_synthetic_timestamp(&self) -> i64 {
        chrono::Utc::now().timestamp()
    }
}

impl MarketProvider for DemoProvider {
    fn get_market(&self, market_id: u64) -> BoxFuture<'_, Result<Option<MarketConfig>, ApiError>> {
        Box::pin(async move { Ok(self.markets.get(&market_id).cloned()) })
    }

    fn list_markets(&self) -> BoxFuture<'_, Result<Vec<MarketConfig>, ApiError>> {
        Box::pin(async move { Ok(self.markets.values().cloned().collect()) })
    }
}

impl PositionProvider for DemoProvider {
    fn get_position<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<Option<PositionSnapshot>, ApiError>> {
        Box::pin(async move { Ok(self.positions.get(id).cloned()) })
    }

    fn list_positions<'a>(
        &'a self,
        owner: Option<&'a str>,
        protocol: Option<&'a str>,
        _network: Option<&'a str>,
    ) -> BoxFuture<'a, Result<Vec<PositionSnapshot>, ApiError>> {
        Box::pin(async move {
            let mut list: Vec<PositionSnapshot> = self
                .positions
                .values()
                .filter(|p| {
                    if let Some(o) = owner {
                        if p.owner != o {
                            return false;
                        }
                    }
                    if let Some(proto) = protocol {
                        if format!("{:?}", p.protocol_source) != proto {
                            return false;
                        }
                    }
                    true
                })
                .cloned()
                .collect();

            list.sort_by(|a, b| a.id.cmp(&b.id));
            Ok(list)
        })
    }
}

impl OracleProvider for DemoProvider {
    fn get_price<'a>(
        &'a self,
        asset_id: &'a AssetId,
    ) -> BoxFuture<'a, Result<Option<Price>, ApiError>> {
        Box::pin(async move {
            let now = self.current_synthetic_timestamp();

            match asset_id {
                AssetId::Sol | AssetId::SSol => Ok(Some(
                    Price::new(150_00000000, -8, 1_00000000, now, 100).with_asset(asset_id.clone()),
                )),
                AssetId::Usdc | AssetId::SUsdc => Ok(Some(
                    Price::new(1_00000000, -8, 1000000, now, 100).with_asset(asset_id.clone()),
                )),
                AssetId::Custom(_) => Ok(None),
            }
        })
    }
}
