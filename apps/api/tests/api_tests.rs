use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use lend_rescue_api::config::AppConfig;
use lend_rescue_api::dto::*;
use lend_rescue_api::router::create_router;
use lend_rescue_api::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

async fn test_app() -> axum::Router {
    let config = AppConfig::for_test();
    let state = AppState::create(config)
        .await
        .expect("Failed to create state");
    create_router(state)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: HealthResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.status, "ok");
    assert_eq!(body.service, "lend-rescue-api");
}

#[tokio::test]
async fn test_ready_endpoint() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: ReadyResponse = serde_json::from_slice(&bytes).unwrap();
    assert!(body.ready);
    assert_eq!(body.database, "in_memory_fallback");
    assert!(body.demo_mode);
}

#[tokio::test]
async fn test_markets_endpoint() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/markets")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let markets: Vec<MarketResponse> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].market_id, 1);
    assert_eq!(markets[0].collateral_asset, "sSOL");
    assert_eq!(markets[0].debt_asset, "sUSDC");
    assert_eq!(markets[0].liquidation_threshold_bps, 8500);
}

#[tokio::test]
async fn test_positions_listing_and_filter() {
    let app = test_app().await;

    // List all
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/positions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let positions: Vec<PositionResponse> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(positions.len(), 4);

    // Filter by owner
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/positions?owner=demo_user_22222222222222222222222222222222")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let filtered: Vec<PositionResponse> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "demo-pos-critical");
}

#[tokio::test]
async fn test_position_detail_and_404() {
    let app = test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/positions/demo-pos-critical")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let pos: PositionResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(pos.id, "demo-pos-critical");
    assert_eq!(pos.collateral_amount.formatted, "10.000000000");
    assert_eq!(pos.debt_amount.formatted, "1200.000000");

    // Unknown position -> 404
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/positions/unknown-position-xyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let err: ErrorResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(err.error.code, "RESOURCE_NOT_FOUND");
}

#[tokio::test]
async fn test_position_risk_parity_with_engine() {
    let app = test_app().await;

    // Golden Scenario 1: 10 sSOL @ $150 ($1,500), 1,200 sUSDC ($1,200), LT = 85%
    // Expected HF = 1.0625, Critical risk band
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/positions/demo-pos-critical/risk")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let risk: RiskAssessmentResponse = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(risk.health_factor.as_deref(), Some("1.0625"));
    assert_eq!(risk.health_factor_scaled, Some(10625));
    assert_eq!(risk.risk_band, "Critical");
    assert_eq!(risk.collateral_value_formatted, "1500.000000");
    assert_eq!(risk.debt_value_formatted, "1200.000000");
    assert_eq!(risk.borrow_capacity_formatted, "75.000000");
    assert!(risk.distance_to_liquidation_bps.unwrap() > 0);

    // Liquidatable position
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/positions/demo-pos-liquidatable/risk")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let liq_risk: RiskAssessmentResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(liq_risk.risk_band, "Liquidatable");
    assert!(liq_risk.health_factor_scaled.unwrap() < 10000);
}

#[tokio::test]
async fn test_oracles_endpoint() {
    let app = test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/oracles/sSOL")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let oracle: OraclePriceResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(oracle.asset_id, "sSOL");
    assert_eq!(oracle.price.formatted, "150.00000000");
    assert!(oracle.is_valid);
    assert!(!oracle.is_stale);

    // Unknown asset -> 404
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/oracles/UNKNOWN_TOKEN")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_recovery_quote_endpoint() {
    let app = test_app().await;

    // Quote RepayDebt for demo-pos-critical to reach HF 1.25
    let req_body = json!({
        "positionId": "demo-pos-critical",
        "action": "RepayDebt",
        "targetHealthFactor": "1.25",
        "maxDebtRepayment": "1000000000",
        "maxCollateralSpend": "5000000000",
        "maxSlippageBps": 100
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/recovery/quote")
                .header("content-type", "application/json")
                .body(Body::from(req_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let plan: RecoveryPlanDto = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(plan.action, "RepayDebt");
    assert_eq!(plan.debt_repaid.formatted, "180.000000");
    assert!(plan.is_feasible);
    assert!(plan.post_health_factor_scaled >= 12500);
}

#[tokio::test]
async fn test_recovery_optimize_and_history_persistence() {
    let app = test_app().await;

    // Optimize recovery for demo-pos-critical
    let req_body = json!({
        "positionId": "demo-pos-critical",
        "targetHealthFactor": "1.25"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/recovery/optimize")
                .header("content-type", "application/json")
                .body(Body::from(req_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let rec: RecoveryRecommendationResponse = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(rec.status, "Quoted");
    assert_eq!(rec.pre_health_factor, "1.0625");
    assert_eq!(rec.target_health_factor, "1.2500");
    assert!(rec.recommended_plan.is_some());
    // Under least-cost objective, RepayDebt ($180.00) is ranked first
    assert_eq!(rec.recommended_plan.as_ref().unwrap().action, "RepayDebt");
    assert_eq!(rec.alternative_plans.len(), 3);

    let recovery_id = rec.recovery_id;

    // Fetch by recovery_id
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/recovery/{}", recovery_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let fetched: RecoveryRecommendationResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(fetched.recovery_id, recovery_id);
    assert_eq!(fetched.position_id, "demo-pos-critical");

    // Invalid UUID -> 400
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/recovery/not-a-valid-uuid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_openapi_json_endpoint() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let openapi: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(openapi["info"]["title"], "LendRescue REST API");
    assert!(openapi["paths"]["/api/v1/positions"].is_object());
    assert!(openapi["paths"]["/api/v1/recovery/optimize"].is_object());
}

#[tokio::test]
async fn test_request_id_middleware_propagation() {
    let app = test_app().await;

    let custom_id = "test-request-id-12345";
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .header("x-request-id", custom_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let header_val = response
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(header_val, custom_id);
}
