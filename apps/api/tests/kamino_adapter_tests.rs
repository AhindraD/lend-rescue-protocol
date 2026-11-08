use std::sync::Arc;

use domain_types::{AssetId, ProtocolSource};
use lend_rescue_api::adapters::kamino::{
    KaminoAdapterConfig, KaminoObligationResponseJson, KaminoPositionProvider, KaminoTransport,
    MockKaminoTransport,
};
use lend_rescue_api::error::ApiError;
use lend_rescue_api::providers::PositionProvider;
use lend_rescue_api::services::{PositionService, RiskService};

#[tokio::test]
async fn test_kamino_position_provider_multi_asset_normalization() {
    let transport = Arc::new(MockKaminoTransport::new());
    let config = KaminoAdapterConfig::default();
    let provider = KaminoPositionProvider::new(config, transport);

    // Query multi-asset obligation
    let positions = provider
        .list_positions(
            Some("kamino_user_multi1111111111111111111111111"),
            Some("kamino"),
            None,
        )
        .await
        .expect("should succeed");

    assert_eq!(positions.len(), 1);
    let pos = &positions[0];

    assert_eq!(pos.id, "kamino-obl-multi-asset");
    assert_eq!(pos.owner, "kamino_user_multi1111111111111111111111111");
    assert_eq!(pos.protocol_source, ProtocolSource::SurfpoolSimulated);

    // Verify all multi-asset collateral deposits are preserved (SOL and bSOL)
    assert_eq!(pos.collateral_positions.len(), 2);
    assert_eq!(pos.collateral_positions[0].asset_symbol, "SOL");
    assert_eq!(pos.collateral_positions[0].decimals, 9);
    assert_eq!(pos.collateral_positions[0].amount.raw, 10_000_000_000);
    assert_eq!(
        pos.collateral_positions[0].market_value_usd_scaled,
        1_500_000_000
    );

    assert_eq!(pos.collateral_positions[1].asset_symbol, "bSOL");
    assert_eq!(pos.collateral_positions[1].decimals, 9);
    assert_eq!(pos.collateral_positions[1].amount.raw, 5_000_000_000);
    assert_eq!(
        pos.collateral_positions[1].market_value_usd_scaled,
        850_000_000
    );

    // Verify all multi-asset debt borrows are preserved (USDC and USDT)
    assert_eq!(pos.debt_positions.len(), 2);
    assert_eq!(pos.debt_positions[0].asset_symbol, "USDC");
    assert_eq!(pos.debt_positions[0].decimals, 6);
    assert_eq!(pos.debt_positions[0].amount.raw, 1_000_000_000);
    assert_eq!(pos.debt_positions[0].market_value_usd_scaled, 1_000_000_000);

    assert_eq!(pos.debt_positions[1].asset_symbol, "USDT");
    assert_eq!(pos.debt_positions[1].decimals, 6);
    assert_eq!(pos.debt_positions[1].amount.raw, 500_000_000);
    assert_eq!(pos.debt_positions[1].market_value_usd_scaled, 500_000_000);

    // Verify official protocol metrics are preserved
    let metrics = pos.protocol_metrics.as_ref().expect("metrics present");
    assert!(metrics.is_protocol_reported);
    assert_eq!(metrics.total_deposit_usd_scaled, 2_350_000_000);
    assert_eq!(metrics.total_borrow_usd_scaled, 1_500_000_000);
    assert_eq!(metrics.borrow_limit_usd_scaled, 1_762_500_000);
    assert_eq!(metrics.liquidation_threshold_usd_scaled, 1_997_500_000);
    assert_eq!(metrics.loan_to_value_bps, 6382);
    assert_eq!(metrics.liquidation_threshold_bps, 8500);
    assert_eq!(metrics.borrow_utilization_bps, 8510);
    assert_eq!(metrics.net_account_value_usd_scaled, 850_000_000);
}

#[tokio::test]
async fn test_kamino_position_provider_healthy_and_critical_scenarios() {
    let transport = Arc::new(MockKaminoTransport::new());
    let config = KaminoAdapterConfig::default();
    let provider = KaminoPositionProvider::new(config, transport);

    // Healthy position
    let healthy_list = provider
        .list_positions(
            Some("kamino_user_1111111111111111111111111111111"),
            Some("kamino"),
            None,
        )
        .await
        .unwrap();

    assert_eq!(healthy_list.len(), 1);
    let healthy = &healthy_list[0];
    assert_eq!(healthy.id, "kamino-obl-healthy");
    let h_metrics = healthy.protocol_metrics.as_ref().unwrap();
    assert_eq!(h_metrics.borrow_utilization_bps, 5925);
    assert_eq!(h_metrics.loan_to_value_bps, 4444);

    // Critical position
    let critical_list = provider
        .list_positions(
            Some("kamino_user_2222222222222222222222222222222"),
            Some("kamino"),
            None,
        )
        .await
        .unwrap();

    assert_eq!(critical_list.len(), 1);
    let critical = &critical_list[0];
    assert_eq!(critical.id, "kamino-obl-critical");
    let c_metrics = critical.protocol_metrics.as_ref().unwrap();
    assert_eq!(c_metrics.loan_to_value_bps, 8000);
    assert_eq!(c_metrics.borrow_utilization_bps, 10666); // > 100% borrow limit
}

#[tokio::test]
async fn test_kamino_position_provider_owner_mismatch_and_filtering() {
    let transport = Arc::new(MockKaminoTransport::new());
    let config = KaminoAdapterConfig::default();
    let provider = KaminoPositionProvider::new(config, transport);

    // Query non-existent owner
    let empty = provider
        .list_positions(Some("unknown_owner"), Some("kamino"), None)
        .await
        .unwrap();
    assert!(empty.is_empty());

    // Filter by different protocol name (e.g. "synthetix") returns empty
    let diff_proto = provider
        .list_positions(
            Some("kamino_user_1111111111111111111111111111111"),
            Some("solend"),
            None,
        )
        .await
        .unwrap();
    assert!(diff_proto.is_empty());
}

#[tokio::test]
async fn test_kamino_transport_timeout_handling() {
    let transport = Arc::new(MockKaminoTransport::new());
    transport.set_timeout(true);

    let config = KaminoAdapterConfig::default();
    let provider = KaminoPositionProvider::new(config, transport);

    let err = provider
        .list_positions(Some("owner"), Some("kamino"), None)
        .await
        .expect_err("should time out");

    match err {
        ApiError::Timeout(msg) => {
            assert!(msg.contains("timeout") || msg.contains("Mock timeout"));
        }
        other => panic!("expected ApiError::Timeout, got {:?}", other),
    }
}

#[tokio::test]
async fn test_kamino_transport_error_handling() {
    let transport = Arc::new(MockKaminoTransport::new());
    transport
        .set_error(Some("Internal RPC connection dropped".to_string()))
        .await;

    let config = KaminoAdapterConfig::default();
    let provider = KaminoPositionProvider::new(config, transport);

    let err = provider
        .list_positions(Some("owner"), Some("kamino"), None)
        .await
        .expect_err("should fail with error");

    match err {
        ApiError::Internal(msg) => {
            assert!(msg.contains("Internal RPC connection dropped"));
        }
        other => panic!("expected ApiError::Internal, got {:?}", other),
    }
}

#[tokio::test]
async fn test_kamino_market_and_reserve_reads() {
    let transport = Arc::new(MockKaminoTransport::new());
    let market_id = "7u3HeHxatcAACvdFtHGryaZWwZxKhqWisdHJaJBYbugj";

    let market = transport
        .get_market(market_id)
        .await
        .unwrap()
        .expect("market should exist");

    assert_eq!(market.market_id, market_id);
    assert_eq!(market.reserves.len(), 2);

    let sol_res = &market.reserves[0];
    assert_eq!(sol_res.token_symbol, "SOL");
    assert_eq!(sol_res.loan_to_value_bps, 7500);
    assert_eq!(sol_res.liquidation_threshold_bps, 8500);
    assert_eq!(sol_res.borrow_interest_rate_bps, 850);
    assert_eq!(sol_res.supply_interest_rate_bps, 425);

    let usdc_res = &market.reserves[1];
    assert_eq!(usdc_res.token_symbol, "USDC");
    assert_eq!(usdc_res.loan_to_value_bps, 8000);
}

#[tokio::test]
async fn test_kamino_risk_service_integration_preserves_protocol_metrics() {
    let transport = Arc::new(MockKaminoTransport::new());
    let config = KaminoAdapterConfig::default();
    let kamino_provider = Arc::new(KaminoPositionProvider::new(config, transport));

    let pool = lend_rescue_api::persistence::DatabasePool::init(None, true)
        .await
        .unwrap();
    let snap_repo = lend_rescue_api::persistence::SnapshotRepository::new(pool);
    let position_service = PositionService::new(kamino_provider.clone(), snap_repo);

    let demo_oracle = Arc::new(lend_rescue_api::providers::DemoProvider::new());
    let risk_service = RiskService::new(position_service, demo_oracle);

    // Query obligations for user 1
    let positions = kamino_provider
        .list_positions(
            Some("kamino_user_1111111111111111111111111111111"),
            Some("kamino"),
            None,
        )
        .await
        .unwrap();
    let pos = &positions[0];

    // Assess risk on position
    let assessment = risk_service
        .assess_position_risk(&pos.id)
        .await
        .expect("risk assessment should succeed");

    assert_eq!(assessment.position_id, pos.id);
    assert_eq!(assessment.data_provenance, "Surfpool Simulated");

    // Ensure protocol-reported metrics are attached to the risk assessment response
    let proto_metrics = assessment
        .protocol_metrics
        .as_ref()
        .expect("protocol metrics should be preserved");

    assert!(proto_metrics.is_protocol_reported);
    assert_eq!(proto_metrics.loan_to_value_bps, 4444);
    assert_eq!(proto_metrics.borrow_utilization_bps, 5925);
    assert_eq!(proto_metrics.total_deposit_usd, "2250.000000");
    assert_eq!(proto_metrics.total_borrow_usd, "1000.000000");
}

#[tokio::test]
async fn test_kamino_obligation_json_deserialization_fidelity() {
    let raw_json = r#"{
        "obligationId": "ObligationTest1111111111111111111111111111",
        "owner": "OwnerTest11111111111111111111111111111111111",
        "marketId": "MarketTest111111111111111111111111111111111",
        "observationSlot": 290123456,
        "observedAt": "2026-11-08T11:00:00Z",
        "protocolSource": "KaminoMainnet",
        "deposits": [
            {
                "reserveAddress": "ReserveSOL",
                "mintAddress": "MintSOL",
                "tokenSymbol": "SOL",
                "decimals": 9,
                "rawAmount": "25000000000",
                "marketValueUsd": "3750.500000"
            }
        ],
        "borrows": [
            {
                "reserveAddress": "ReserveUSDC",
                "mintAddress": "MintUSDC",
                "tokenSymbol": "USDC",
                "decimals": 6,
                "rawAmount": "1500000000",
                "marketValueUsd": "1500.000000"
            }
        ],
        "protocolMetrics": {
            "borrowLimitUsd": "2812.875000",
            "liquidationThresholdUsd": "3187.925000",
            "totalDepositUsd": "3750.500000",
            "totalBorrowUsd": "1500.000000",
            "loanToValueBps": 3999,
            "liquidationThresholdBps": 8500,
            "borrowUtilizationBps": 5332,
            "netAccountValueUsd": "2250.500000",
            "isProtocolReported": true
        }
    }"#;

    let parsed: KaminoObligationResponseJson =
        serde_json::from_str(raw_json).expect("should parse JSON");

    assert_eq!(
        parsed.obligation_id,
        "ObligationTest1111111111111111111111111111"
    );
    assert_eq!(parsed.protocol_source, "KaminoMainnet");

    let domain = parsed.to_domain().expect("should map to domain snapshot");
    assert_eq!(domain.protocol_source, ProtocolSource::KaminoMainnet);
    assert_eq!(domain.observation_slot, 290123456);
    assert_eq!(domain.deposits[0].raw_amount.raw, 25_000_000_000);
    assert_eq!(domain.deposits[0].decimals, 9);
    assert_eq!(domain.deposits[0].market_value_usd_scaled, 3_750_500_000);

    let pos = parsed
        .to_position_snapshot()
        .expect("should map to PositionSnapshot");
    assert_eq!(pos.collateral_asset, AssetId::SSol);
    assert_eq!(pos.debt_asset, AssetId::SUsdc);
    assert_eq!(pos.collateral_amount.raw, 25_000_000_000);
    assert_eq!(pos.debt_amount.raw, 1_500_000_000);
    assert_eq!(pos.protocol_source.label(), "Mainnet Observed");
}
