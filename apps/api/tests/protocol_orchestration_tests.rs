use std::env;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use domain_types::{AssetId, HealthFactor, PositionSnapshot, TokenAmount};
use http_body_util::BodyExt;
use lend_rescue_api::adapters::jupiter::{
    JupiterConfig, JupiterSwapQuoteProvider, MockJupiterTransport,
};
use lend_rescue_api::adapters::pyth::{
    try_decode_price_feed_account, try_decode_pyth_account, AccountDataResult,
    HttpSolanaRpcTransport, MockSolanaRpcTransport, PriceFeedMessage, PriceUpdateV2,
    PythAccountKind, PythCoreConfig, PythOracleProvider, SolanaRpcTransport, VerificationLevel,
    DEFAULT_SOL_USD_FEED_ACCOUNT, DEFAULT_USDC_USD_FEED_ACCOUNT, PRICE_UPDATE_V2_DISCRIMINATOR,
    PYTH_RECEIVER_PROGRAM_ID,
};
use lend_rescue_api::config::AppConfig;
use lend_rescue_api::dto::ErrorResponse;
use lend_rescue_api::providers::{OracleProvider, OracleSource};
use lend_rescue_api::router::create_router;
use lend_rescue_api::state::AppState;
use recovery_engine::optimizer::generate_recovery_candidates;
use recovery_engine::recovery_cost::SwapPoolParams;
use risk_engine::health_factor::compute_health_factor;
use tower::ServiceExt;

const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

/// Helper: creates a mock PriceUpdateV2 account bytes buffer.
fn create_mock_price_update_bytes(
    feed_id: [u8; 32],
    price: i64,
    conf: u64,
    expo: i32,
    publish_time: i64,
) -> Vec<u8> {
    let mut data = Vec::with_capacity(146);
    data.extend_from_slice(&PRICE_UPDATE_V2_DISCRIMINATOR);
    let update = PriceUpdateV2 {
        write_authority: [0u8; 32],
        verification_level: VerificationLevel::Full,
        price_message: PriceFeedMessage {
            feed_id,
            price,
            conf,
            expo,
            publish_time,
            prev_publish_time: publish_time - 1,
            ema_price: price,
            ema_conf: conf,
        },
        posted_slot: 466900000,
    };
    borsh::BorshSerialize::serialize(&update, &mut data).unwrap();
    data
}

// ---------------------------------------------------------------------------
// 1. Pyth Account-Model & Persistent Feed Verification
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_pyth_account_model_distinction_and_decoding() {
    let sol_feed_id =
        hex::decode("ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d").unwrap();
    let mut feed_id = [0u8; 32];
    feed_id.copy_from_slice(&sol_feed_id);

    let now = Utc::now().timestamp();
    let account_bytes = create_mock_price_update_bytes(feed_id, 185_5000_0000, 20_0000, -8, now);

    // 1. Generic try_decode_pyth_account identifies as PriceUpdate
    let generic = try_decode_pyth_account(&account_bytes).expect("decoding should succeed");
    assert_eq!(generic.account_kind, PythAccountKind::PriceUpdate);
    assert_eq!(generic.feed_id, Some(feed_id));
    assert_eq!(generic.price, 185_5000_0000);
    assert_eq!(generic.expo, -8);
    assert!(!generic.is_legacy);

    // 2. Persistent feed decoder identifies as PriceFeed
    let feed_decoded =
        try_decode_price_feed_account(&account_bytes).expect("feed decoding should succeed");
    assert_eq!(feed_decoded.account_kind, PythAccountKind::PriceFeed);
    assert_eq!(feed_decoded.feed_id, Some(feed_id));
    assert_eq!(feed_decoded.price, 185_5000_0000);

    // 3. Persistent PDA addresses match Pyth standard
    assert_eq!(
        DEFAULT_SOL_USD_FEED_ACCOUNT,
        "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"
    );
    assert_eq!(
        DEFAULT_USDC_USD_FEED_ACCOUNT,
        "Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX"
    );
}

#[tokio::test]
async fn test_pyth_provider_with_mock_rpc_preserves_provenance() {
    let sol_feed_id =
        hex::decode("ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d").unwrap();
    let mut feed_id = [0u8; 32];
    feed_id.copy_from_slice(&sol_feed_id);

    let now = Utc::now().timestamp();
    let account_bytes = create_mock_price_update_bytes(feed_id, 190_0000_0000, 10_0000, -8, now);

    let rpc = Arc::new(MockSolanaRpcTransport::new());
    rpc.set_account(
        DEFAULT_SOL_USD_FEED_ACCOUNT,
        AccountDataResult {
            owner: PYTH_RECEIVER_PROGRAM_ID.to_string(),
            data: account_bytes,
            slot: 466900000,
        },
    )
    .await;

    let config = PythCoreConfig::new("http://127.0.0.1:8899", "surfpool");
    let provider = PythOracleProvider::new(config, rpc, None);

    let obs = provider
        .get_observation(&AssetId::Sol)
        .await
        .expect("observation lookup should succeed")
        .expect("observation should exist");

    assert_eq!(obs.source, OracleSource::SurfpoolSimulated);
    assert_eq!(obs.source.as_str(), "Surfpool Simulated");
    assert!(obs.is_valid);
    assert_eq!(obs.price.mantissa, 190_0000_0000);
    assert_eq!(obs.price.exponent, -8);
    assert_eq!(obs.slot, Some(466900000));
}

// ---------------------------------------------------------------------------
// 2. Cross-Protocol Orchestration Test (Offline / Synthetic Golden Path)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_cross_protocol_orchestration_full_pipeline() {
    // Pipeline: Kamino Position -> Pyth Oracle -> Risk Engine -> Recovery Feasibility -> Jupiter Quote -> Advisory Plan

    // 1. Single-pair SOL-USDC critical position
    let position = PositionSnapshot::synthetic_demo(
        "pos-golden-1",
        "wallet-golden-1",
        1_000_000_000, // 1.0 SOL
        125_000_000,   // 125.0 USDC
        8500,          // 85% liquidation threshold
    );

    // 2. Oracle valuation from Pyth Core
    let sol_feed_id =
        hex::decode("ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d").unwrap();
    let mut feed_id = [0u8; 32];
    feed_id.copy_from_slice(&sol_feed_id);

    let now = Utc::now().timestamp();
    let sol_account_bytes =
        create_mock_price_update_bytes(feed_id, 150_0000_0000, 10_0000, -8, now);

    let usdc_feed_id =
        hex::decode("eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a").unwrap();
    let mut u_feed_id = [0u8; 32];
    u_feed_id.copy_from_slice(&usdc_feed_id);
    let usdc_account_bytes =
        create_mock_price_update_bytes(u_feed_id, 1_0000_0000, 1_0000, -8, now);

    let rpc = Arc::new(MockSolanaRpcTransport::new());
    rpc.set_account(
        DEFAULT_SOL_USD_FEED_ACCOUNT,
        AccountDataResult {
            owner: PYTH_RECEIVER_PROGRAM_ID.to_string(),
            data: sol_account_bytes,
            slot: 466900100,
        },
    )
    .await;
    rpc.set_account(
        DEFAULT_USDC_USD_FEED_ACCOUNT,
        AccountDataResult {
            owner: PYTH_RECEIVER_PROGRAM_ID.to_string(),
            data: usdc_account_bytes,
            slot: 466900100,
        },
    )
    .await;

    let pyth_config = PythCoreConfig::new("http://127.0.0.1:8899", "surfpool");
    let oracle_provider = Arc::new(PythOracleProvider::new(pyth_config, rpc, None));

    let sol_price = oracle_provider
        .get_price(&AssetId::Sol)
        .await
        .unwrap()
        .unwrap();
    let usdc_price = oracle_provider
        .get_price(&AssetId::Usdc)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(sol_price.mantissa, 150_0000_0000);
    assert_eq!(usdc_price.mantissa, 1_0000_0000);

    // 3. Risk Engine assessment: HF = (1.0 * 150 * 0.85) / 125 = 127.5 / 125 = 1.0200 = 10,200 scaled
    let collateral_amt = TokenAmount::new(1_000_000_000, 9, Some(AssetId::Sol));
    let debt_amt = TokenAmount::new(125_000_000, 6, Some(AssetId::Usdc));
    let hf = compute_health_factor(&collateral_amt, &sol_price, &debt_amt, &usdc_price, 8500)
        .expect("HF calculation should succeed");

    assert_eq!(hf, HealthFactor(10_200));

    // 4. Jupiter Quote Provider (indicative quote-only)
    let jup_transport = Arc::new(MockJupiterTransport::new());
    let jup_provider = Arc::new(JupiterSwapQuoteProvider::new(
        JupiterConfig::default_v2(),
        jup_transport.clone(),
    ));

    let quote = jup_provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 100, false)
        .await
        .expect("jupiter quote should succeed");

    assert_eq!(quote.in_amount.raw, 1_000_000_000);
    assert_eq!(quote.expected_out_amount.raw, 150_000_000);
    assert!(quote.is_indicative);

    // 5. Recovery Optimizer candidate ranking
    let pool = SwapPoolParams::new(10_000_000_000_000, 1_500_000_000_000, 30);
    let target_hf = HealthFactor(12_500); // Target HF 1.25
    let candidates = generate_recovery_candidates(
        &position,
        &sol_price,
        &usdc_price,
        target_hf,
        Some(&pool),
        None,
        now,
    )
    .expect("optimizer candidate generation should succeed");

    assert!(!candidates.is_empty());
    let top_candidate = &candidates[0];
    assert!(top_candidate.is_feasible);
    assert!(top_candidate.post_health_factor.0 >= 12_500);

    // 6. Security invariant: Taker was NEVER passed to Jupiter
    let captured = jup_transport.get_requests().await;
    assert_eq!(captured.len(), 1);
    assert!(!captured[0].has_taker);
}

// ---------------------------------------------------------------------------
// 3. Multi-Asset Safety Guard Invariant
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_multi_asset_kamino_safety_guard_rejects_recovery_optimization() {
    let mut app_config = AppConfig::for_test();
    app_config.position_source = "kamino_mock".to_string();

    let state = AppState::create(app_config).await.unwrap();
    let router = create_router(state);

    // Attempting recovery optimization on multi-asset Kamino position must fail-closed with 422
    let payload = serde_json::json!({
        "positionId": "kamino-obl-multi-asset",
        "targetHealthFactor": "1.25"
    });

    let resp = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/recovery/optimize")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let err: ErrorResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(err.error.code, "UNPROCESSABLE_ENTITY");
    assert!(err
        .error
        .message
        .contains("unsupported for multi-asset Kamino positions"));
}

// ---------------------------------------------------------------------------
// 4. Live Surfpool Mainnet Fork Integration Tests (Conditional)
// ---------------------------------------------------------------------------

/// Probe helper to check if Surfpool local RPC is active on 127.0.0.1:8899.
async fn check_surfpool_live() -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .unwrap_or_default();

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getSlot"
    });

    match client
        .post("http://127.0.0.1:8899")
        .json(&body)
        .send()
        .await
    {
        Ok(res) => res.status().is_success(),
        Err(_) => false,
    }
}

#[tokio::test]
async fn test_live_surfpool_pyth_account_read_and_validation() {
    let is_live = env::var("SURFPOOL_LIVE").is_ok() || check_surfpool_live().await;
    if !is_live {
        eprintln!("[INFO] Skipping test_live_surfpool_pyth_account_read_and_validation: Surfpool not active on 127.0.0.1:8899");
        return;
    }

    // 1. Connect HttpSolanaRpcTransport to Surfpool
    let rpc_transport = Arc::new(HttpSolanaRpcTransport::new("http://127.0.0.1:8899", 5000));

    // 2. Query persistent SOL/USD feed account
    let sol_account = rpc_transport
        .get_account_data(DEFAULT_SOL_USD_FEED_ACCOUNT)
        .await
        .expect("RPC read should succeed")
        .expect("SOL/USD persistent feed account must exist on Surfpool fork");

    // 3. Verify owner is Pyth receiver program
    assert_eq!(sol_account.owner, PYTH_RECEIVER_PROGRAM_ID);
    assert!(!sol_account.data.is_empty());

    // 4. Decode using persistent price feed decoder
    let decoded =
        try_decode_price_feed_account(&sol_account.data).expect("Price feed decoding must succeed");

    assert_eq!(decoded.account_kind, PythAccountKind::PriceFeed);
    assert!(
        decoded.price > 0,
        "SOL price mantissa must be positive: {}",
        decoded.price
    );
    assert_eq!(decoded.expo, -8);
    assert!(decoded.conf > 0, "Confidence interval must be non-negative");

    // 5. Query persistent USDC/USD feed account
    let usdc_account = rpc_transport
        .get_account_data(DEFAULT_USDC_USD_FEED_ACCOUNT)
        .await
        .expect("RPC read should succeed")
        .expect("USDC/USD persistent feed account must exist on Surfpool fork");

    assert_eq!(usdc_account.owner, PYTH_RECEIVER_PROGRAM_ID);
    let usdc_decoded = try_decode_price_feed_account(&usdc_account.data)
        .expect("USDC price feed decoding must succeed");

    assert_eq!(usdc_decoded.account_kind, PythAccountKind::PriceFeed);
    assert!(
        usdc_decoded.price > 0,
        "USDC price mantissa must be positive: {}",
        usdc_decoded.price
    );
}

#[tokio::test]
async fn test_live_surfpool_kamino_market_read() {
    let is_live = env::var("SURFPOOL_LIVE").is_ok() || check_surfpool_live().await;
    if !is_live {
        eprintln!("[INFO] Skipping test_live_surfpool_kamino_market_read: Surfpool not active on 127.0.0.1:8899");
        return;
    }

    // Direct account check on Surfpool for Kamino Main Market
    let rpc_transport = Arc::new(HttpSolanaRpcTransport::new("http://127.0.0.1:8899", 5000));
    let main_market = "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF";

    let market_account = rpc_transport
        .get_account_data(main_market)
        .await
        .expect("Kamino market read should succeed")
        .expect("Kamino Main Market must exist on Surfpool fork");

    assert_eq!(
        market_account.owner,
        "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD"
    );
    assert_eq!(market_account.data.len(), 4664);
}

#[tokio::test]
async fn test_live_kamino_obligation_read() {
    let smoke_wallet = env::var("KAMINO_SMOKE_WALLET").ok();
    match smoke_wallet {
        Some(wallet) if !wallet.trim().is_empty() => {
            let kamino_url = env::var("KAMINO_ADAPTER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:3002".to_string());
            let market = env::var("KAMINO_MARKET_ADDRESS")
                .or_else(|_| env::var("KAMINO_DEFAULT_MARKET"))
                .unwrap_or_else(|_| "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF".to_string());

            let client = reqwest::Client::builder()
                .timeout(Duration::from_millis(5000))
                .build()
                .unwrap();

            let url = format!("{}/obligations/{}/{}", kamino_url, market, wallet);
            if let Ok(resp) = client.get(&url).send().await {
                if resp.status().is_success() {
                    let json: serde_json::Value = resp.json().await.unwrap();
                    println!(
                        "[LIVE KAMINO] Obligation found for {}: {:?}",
                        wallet,
                        json.get("obligationId")
                    );
                } else {
                    println!(
                        "[LIVE KAMINO] Obligation lookup returned status: {}",
                        resp.status()
                    );
                }
            }
        }
        _ => {
            println!("[INFO] KAMINO_SMOKE_WALLET not configured; skipping live obligation lookup without failure");
        }
    }
}

#[tokio::test]
async fn test_live_jupiter_v2_quote_verification() {
    let is_live = env::var("JUPITER_LIVE_TEST")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if !is_live {
        eprintln!("[INFO] Skipping test_live_jupiter_v2_quote_verification (enable with JUPITER_LIVE_TEST=1)");
        return;
    }

    let api_key = env::var("JUPITER_API_KEY").ok();
    let config = JupiterConfig::default_v2()
        .with_api_key(api_key)
        .with_request_timeout_ms(10000);

    let transport =
        Arc::new(lend_rescue_api::adapters::jupiter::JupiterHttpTransport::new(config.clone()));
    let provider = JupiterSwapQuoteProvider::new(config, transport);

    // Small bounded quote: 0.05 SOL (50_000_000 raw lamports) to USDC
    let quote = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 50_000_000, 100, true)
        .await
        .expect("Live Jupiter V2 quote request must succeed");

    assert_eq!(quote.provider, "Jupiter");
    assert_eq!(quote.input_mint, SOL_MINT);
    assert_eq!(quote.output_mint, USDC_MINT);
    assert_eq!(quote.in_amount.raw, 50_000_000);
    assert!(
        quote.expected_out_amount.raw > 0,
        "Output amount must be positive"
    );
    assert!(quote.is_indicative);
}
