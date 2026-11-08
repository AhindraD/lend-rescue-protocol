use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use lend_rescue_api::adapters::jupiter::{
    JupiterConfig, JupiterOrderResponseJson, JupiterSwapQuoteProvider, MockJupiterTransport,
};
use lend_rescue_api::config::AppConfig;
use lend_rescue_api::dto::*;
use lend_rescue_api::error::ApiError;
use lend_rescue_api::router::create_router;
use lend_rescue_api::state::AppState;
use serde_json::json;
use tower::ServiceExt;

const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const BSOL_MINT: &str = "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1";

#[tokio::test]
async fn test_jupiter_request_url_and_quote_only_parameters() {
    let transport = Arc::new(MockJupiterTransport::new());
    let config = JupiterConfig::default_v2().with_quote_ttl_secs(30);
    let provider = JupiterSwapQuoteProvider::new(config, transport.clone());

    let quote = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 100, false)
        .await
        .unwrap();

    assert_eq!(quote.provider, "Jupiter");
    assert_eq!(quote.input_mint, SOL_MINT);
    assert_eq!(quote.output_mint, USDC_MINT);
    assert_eq!(quote.in_amount.raw, 1_000_000_000);
    assert_eq!(quote.expected_out_amount.raw, 150_000_000);
    assert!(quote.is_indicative);

    // Verify recorded transport request
    let reqs = transport.get_requests().await;
    assert_eq!(reqs.len(), 1);
    let req = &reqs[0];

    // 1. Correct URL and endpoint
    assert!(req.url.starts_with("https://api.jup.ag/swap/v2/order"));
    assert!(req.url.contains(&format!("inputMint={}", SOL_MINT)));
    assert!(req.url.contains(&format!("outputMint={}", USDC_MINT)));
    assert!(req.url.contains("amount=1000000000"));

    // 2. Critical Quote-Only Invariant: taker parameter is NEVER present
    assert!(!req.url.contains("taker"));
    assert!(!req.url.contains("payer"));
    assert!(!req.has_taker);
}

#[tokio::test]
async fn test_jupiter_api_key_header_handling_and_redaction() {
    let transport =
        Arc::new(MockJupiterTransport::new().with_expected_api_key(Some("secret-key".to_string())));
    let config = JupiterConfig::default_v2().with_api_key(Some("secret-key".to_string()));

    // Verify debug redaction: key must NEVER be printed
    let debug_str = format!("{:?}", config);
    assert!(!debug_str.contains("secret-key"));
    assert!(debug_str.contains("[REDACTED]"));

    let provider = JupiterSwapQuoteProvider::new(config, transport.clone());
    let quote = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 100, false)
        .await
        .unwrap();

    assert_eq!(quote.expected_out_amount.raw, 150_000_000);
    let reqs = transport.get_requests().await;
    assert_eq!(reqs.len(), 1);
    assert!(reqs[0].has_api_key);
}

#[tokio::test]
async fn test_quote_only_security_assertion_rejects_unexpected_transaction() {
    let transport = Arc::new(MockJupiterTransport::new());
    transport.set_force_unexpected_transaction(true);

    let config = JupiterConfig::default_v2();
    let provider = JupiterSwapQuoteProvider::new(config, transport);

    let result = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 100, false)
        .await;

    // Must fail with security violation because transaction payload was unexpectedly non-null
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(err.to_string().contains("unexpected transaction payload"));
}

#[tokio::test]
async fn test_jupiter_quote_normalization_bsol_to_usdc() {
    let transport = Arc::new(MockJupiterTransport::new());
    let config = JupiterConfig::default_v2();
    let provider = JupiterSwapQuoteProvider::new(config, transport);

    let quote = provider
        .get_swap_quote(BSOL_MINT, USDC_MINT, 1_000_000_000, 50, false)
        .await
        .unwrap();

    assert_eq!(quote.input_mint, BSOL_MINT);
    assert_eq!(quote.output_mint, USDC_MINT);
    assert_eq!(quote.in_amount.raw, 1_000_000_000);
    assert_eq!(quote.in_amount.decimals, 9);
    assert_eq!(quote.expected_out_amount.raw, 165_000_000);
    assert_eq!(quote.expected_out_amount.decimals, 6);
    assert_eq!(quote.fee_bps, 12);
    assert_eq!(quote.router, "iris");
    assert_eq!(quote.price_impact_pct.as_deref(), Some("0.01"));

    // 50 bps slippage (0.5%): 165_000_000 * 9950 / 10000 = 164_175_000
    assert_eq!(quote.planning_min_out_amount.raw, 164_175_000);
    assert_eq!(quote.route_plan.len(), 1);
    assert_eq!(quote.route_plan[0].label.as_deref(), Some("Orca Whirlpool"));
}

#[tokio::test]
async fn test_fee_accounting_does_not_double_deduct() {
    // In Jupiter V2, outAmount already deducts platform fees.
    // Ensure our planning floor and expected out amount never subtract fees twice.
    let transport = Arc::new(MockJupiterTransport::new());
    let config = JupiterConfig::default_v2();
    let provider = JupiterSwapQuoteProvider::new(config, transport);

    let quote = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 100, false)
        .await
        .unwrap();

    // Raw outAmount returned by Jupiter fixture is 150_000_000
    assert_eq!(quote.expected_out_amount.raw, 150_000_000);

    // Slippage planning floor for 100 bps: 150_000_000 * 9900 / 10000 = 148_500_000
    assert_eq!(quote.planning_min_out_amount.raw, 148_500_000);
}

#[tokio::test]
async fn test_slippage_planning_math_and_validation() {
    let transport = Arc::new(MockJupiterTransport::new());
    let config = JupiterConfig::default_v2();
    let provider = JupiterSwapQuoteProvider::new(config, transport);

    // 0 bps slippage: min_out == expected_out
    let quote_zero = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 0, false)
        .await
        .unwrap();
    assert_eq!(quote_zero.planning_min_out_amount.raw, 150_000_000);

    // 10,000 bps slippage (100%): min_out == 0
    let quote_full = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 10_000, true)
        .await
        .unwrap();
    assert_eq!(quote_full.planning_min_out_amount.raw, 0);

    // Invalid slippage > 10,000 bps must be rejected
    let result_invalid = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 10_001, true)
        .await;
    assert!(result_invalid.is_err());
    assert_eq!(
        result_invalid.unwrap_err().status_code(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_large_integer_precision_preserved_without_floating_point() {
    let transport = Arc::new(MockJupiterTransport::new());
    // 2^53 + 1 = 9_007_199_254_740_993 (> Number.MAX_SAFE_INTEGER in JavaScript)
    let large_raw: u64 = 9_007_199_254_740_993;

    transport
        .add_fixture(
            SOL_MINT,
            USDC_MINT,
            large_raw,
            JupiterOrderResponseJson {
                in_amount: large_raw.to_string(),
                out_amount: "1351079888211148".to_string(),
                fee_bps: 10,
                fee_mint: Some(USDC_MINT.to_string()),
                platform_fee: None,
                price_impact_pct: Some("0.05".to_string()),
                route_plan: vec![],
                router: Some("iris".to_string()),
                mode: Some("ExactIn".to_string()),
                transaction: None,
            },
        )
        .await;

    let config = JupiterConfig::default_v2();
    let provider = JupiterSwapQuoteProvider::new(config, transport);

    let quote = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, large_raw, 100, false)
        .await
        .unwrap();

    assert_eq!(quote.in_amount.raw, large_raw);
    assert_eq!(quote.expected_out_amount.raw, 1_351_079_888_211_148);

    // Verify DTO serialization preserves exact raw string
    let dto = SwapQuoteDto::from_domain(&quote);
    assert_eq!(dto.in_amount.raw, "9007199254740993");
    assert_eq!(dto.expected_out_amount.raw, "1351079888211148");
}

#[tokio::test]
async fn test_input_validation_rejections() {
    let transport = Arc::new(MockJupiterTransport::new());
    let config = JupiterConfig::default_v2();
    let provider = JupiterSwapQuoteProvider::new(config, transport);

    // 1. Empty mint
    assert!(provider
        .get_swap_quote("", USDC_MINT, 1000, 100, false)
        .await
        .is_err());

    // 2. Identical mints
    assert!(provider
        .get_swap_quote(SOL_MINT, SOL_MINT, 1000, 100, false)
        .await
        .is_err());

    // 3. Zero amount
    assert!(provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 0, 100, false)
        .await
        .is_err());

    // 4. Invalid base58 characters
    assert!(provider
        .get_swap_quote(
            "Invalid0000000000000000000000000000",
            USDC_MINT,
            1000,
            100,
            false
        )
        .await
        .is_err());
}

#[tokio::test]
async fn test_resilience_rate_limit_429_with_retry_after() {
    let transport = Arc::new(MockJupiterTransport::new());
    transport.set_force_rate_limit(Some(5)).await;

    let config = JupiterConfig::default_v2();
    let provider = JupiterSwapQuoteProvider::new(config, transport);

    let result = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 100, false)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(err.error_code(), "RATE_LIMITED");
    assert!(err.to_string().contains("5s"));
}

#[tokio::test]
async fn test_resilience_authentication_failure_401() {
    let transport = Arc::new(MockJupiterTransport::new());
    transport.set_force_auth_failure(true);

    let config = JupiterConfig::default_v2();
    let provider = JupiterSwapQuoteProvider::new(config, transport);

    let result = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 100, false)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
    assert_eq!(err.error_code(), "UNAUTHORIZED");
}

#[tokio::test]
async fn test_resilience_timeout_handling() {
    let transport = Arc::new(MockJupiterTransport::new());
    transport.set_force_timeout(true);

    let config = JupiterConfig::default_v2();
    let provider = JupiterSwapQuoteProvider::new(config, transport);

    let result = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 100, false)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), StatusCode::GATEWAY_TIMEOUT);
    assert_eq!(err.error_code(), "TIMEOUT");
}

#[tokio::test]
async fn test_resilience_no_route_found() {
    let transport = Arc::new(MockJupiterTransport::new());
    transport.set_force_no_route(true);

    let config = JupiterConfig::default_v2();
    let provider = JupiterSwapQuoteProvider::new(config, transport);

    let result = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 100, false)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_caching_and_ttl_expiration() {
    let transport = Arc::new(MockJupiterTransport::new());
    // TTL = 1 second
    let config = JupiterConfig::default_v2().with_quote_ttl_secs(1);
    let provider = JupiterSwapQuoteProvider::new(config, transport.clone());

    // 1. Initial request hits transport
    let _q1 = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 100, false)
        .await
        .unwrap();
    assert_eq!(transport.get_requests().await.len(), 1);

    // 2. Second request hits in-memory cache without querying transport again
    let _q2 = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 100, false)
        .await
        .unwrap();
    assert_eq!(transport.get_requests().await.len(), 1);

    // 3. Forcing refresh queries transport again
    let _q3 = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 100, true)
        .await
        .unwrap();
    assert_eq!(transport.get_requests().await.len(), 2);
}

#[tokio::test]
async fn test_no_silent_mock_fallback_on_jupiter_failure() {
    // When Jupiter is explicitly configured, failure must surface directly as an error,
    // and must NEVER fall back silently to mock demo pools.
    let transport = Arc::new(MockJupiterTransport::new());
    transport.set_force_rate_limit(Some(10)).await;

    let config = JupiterConfig::default_v2();
    let provider = JupiterSwapQuoteProvider::new(config, transport);

    let result = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 1_000_000_000, 100, false)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::TooManyRequests(msg) => assert!(msg.contains("10s")),
        other => panic!("Expected TooManyRequests error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_api_standalone_jupiter_quote_endpoint() {
    let app_config = AppConfig::for_test();
    let state = AppState::create(app_config).await.unwrap();
    let router = create_router(state);

    let payload = json!({
        "inputMint": SOL_MINT,
        "outputMint": USDC_MINT,
        "amount": "1000000000",
        "maxSlippageBps": 100
    });

    let resp = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/recovery/quote")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let plan: RecoveryPlanDto = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(plan.action, "SwapQuote");
    assert_eq!(plan.action_label, "Indicative Swap Quote");
    assert_eq!(plan.input_amount.raw, "1000000000");
    assert!(plan.expected_output_amount.is_some());
    assert!(plan.min_output_amount.is_some());
    assert!(plan.is_feasible);

    let swap_quote = plan.swap_quote.expect("Expected swap_quote to be present");
    assert_eq!(swap_quote.input_mint, SOL_MINT);
    assert_eq!(swap_quote.output_mint, USDC_MINT);
    assert!(swap_quote.is_indicative);
    assert!(swap_quote.is_valid);
}

#[tokio::test]
async fn test_multi_asset_kamino_safety_guard_rejects_recovery_optimization() {
    let mut app_config = AppConfig::for_test();
    app_config.position_source = "kamino_mock".to_string();

    let state = AppState::create(app_config).await.unwrap();
    let router = create_router(state);

    // Attempting full recovery optimization on a multi-asset Kamino position must be rejected
    // with 422 Unprocessable Entity
    let payload = json!({
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

#[tokio::test]
async fn test_multi_asset_kamino_safety_guard_rejects_synthetic_quote() {
    let mut app_config = AppConfig::for_test();
    app_config.position_source = "kamino_mock".to_string();

    let state = AppState::create(app_config).await.unwrap();
    let router = create_router(state);

    // Attempting synthetic quote on a multi-asset Kamino position must also be rejected
    let payload = json!({
        "positionId": "kamino-obl-multi-asset",
        "action": "RepayDebt"
    });

    let resp = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/recovery/quote")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let err: ErrorResponse = serde_json::from_slice(&bytes).unwrap();
    assert!(err
        .error
        .message
        .contains("unsupported for multi-asset Kamino positions"));
}

#[tokio::test]
async fn test_optional_live_jupiter_quote_smoke_test() {
    // Only executed when explicitly enabled via JUPITER_LIVE_TEST=1
    let live_enabled = std::env::var("JUPITER_LIVE_TEST")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if !live_enabled {
        println!("Skipping live Jupiter quote test (enable with JUPITER_LIVE_TEST=1)");
        return;
    }

    let api_key = std::env::var("JUPITER_API_KEY").ok();
    let config = JupiterConfig::default_v2()
        .with_api_key(api_key)
        .with_request_timeout_ms(10000);

    let transport =
        Arc::new(lend_rescue_api::adapters::jupiter::JupiterHttpTransport::new(config.clone()));
    let provider = JupiterSwapQuoteProvider::new(config, transport);

    // Small illustrative amount: 0.1 SOL (100_000_000 raw) to USDC
    let quote = provider
        .get_swap_quote(SOL_MINT, USDC_MINT, 100_000_000, 100, true)
        .await
        .expect("Live quote request failed");

    assert_eq!(quote.provider, "Jupiter");
    assert_eq!(quote.input_mint, SOL_MINT);
    assert_eq!(quote.output_mint, USDC_MINT);
    assert_eq!(quote.in_amount.raw, 100_000_000);
    assert!(quote.expected_out_amount.raw > 0);
    assert!(quote.planning_min_out_amount.raw > 0);
    assert!(quote.planning_min_out_amount.raw <= quote.expected_out_amount.raw);
    assert!(quote.is_indicative);
    println!(
        "Live Jupiter quote success: 0.1 SOL -> {} USDC (router: {})",
        quote.expected_out_amount.to_decimal_string(),
        quote.router
    );
}
