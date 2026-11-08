use domain_types::{AssetId, Price, TokenAmount};
use lend_rescue_api::dto::*;
use lend_rescue_api::error::ApiError;
use lend_rescue_api::persistence::DatabasePool;
use lend_rescue_api::persistence::{
    ObservationRepository, RecommendationRepository, SnapshotRepository,
};
use lend_rescue_api::services::recovery_service::parse_health_factor;
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_numerical_precision_serialization() {
    // Large 64-bit integer amount that would lose precision in JS Number if serialized as float
    let raw_val: u64 = 9_007_199_254_740_993; // 2^53 + 1
    let token = TokenAmount::new(raw_val, 9, Some(AssetId::Sol));
    let dto = TokenAmountDto::from_domain(&token);

    assert_eq!(dto.raw, "9007199254740993");
    let serialized = serde_json::to_string(&dto).unwrap();
    // Must be serialized as JSON string, NOT a JSON float/number
    assert!(serialized.contains(r#""raw":"9007199254740993""#));

    // Price mantissa serialization
    let price = Price::new(150_00000000, -8, 1_00000000, 1700000000, 100);
    let price_dto = PriceDto::from_domain(&price);
    assert_eq!(price_dto.mantissa, "15000000000");
    assert_eq!(price_dto.formatted, "150.00000000");
    let price_serialized = serde_json::to_string(&price_dto).unwrap();
    assert!(price_serialized.contains(r#""mantissa":"15000000000""#));
}

#[test]
fn test_parse_health_factor_edge_cases() {
    // Standard decimals
    assert_eq!(parse_health_factor(Some("1.25"), 10000).unwrap().0, 12500);
    assert_eq!(parse_health_factor(Some("1.0625"), 10000).unwrap().0, 10625);
    assert_eq!(parse_health_factor(Some("1.0"), 10000).unwrap().0, 10000);
    assert_eq!(parse_health_factor(Some("2.0"), 10000).unwrap().0, 20000);

    // Direct integer bps
    assert_eq!(parse_health_factor(Some("12500"), 10000).unwrap().0, 12500);

    // Default fallback
    assert_eq!(parse_health_factor(None, 11500).unwrap().0, 11500);
    assert_eq!(parse_health_factor(Some("   "), 11500).unwrap().0, 11500);

    // Below 1.0 (sub-solvency forbidden)
    assert!(parse_health_factor(Some("0.9999"), 10000).is_err());
    assert!(parse_health_factor(Some("9999"), 10000).is_err());

    // Too many decimal places (> 4)
    assert!(parse_health_factor(Some("1.12345"), 10000).is_err());
}

#[test]
fn test_api_error_status_mapping() {
    let err_not_found = ApiError::NotFound("position 123".to_string());
    assert_eq!(
        err_not_found.status_code(),
        axum::http::StatusCode::NOT_FOUND
    );
    assert_eq!(err_not_found.error_code(), "RESOURCE_NOT_FOUND");

    let err_bad_req = ApiError::BadRequest("invalid format".to_string());
    assert_eq!(
        err_bad_req.status_code(),
        axum::http::StatusCode::BAD_REQUEST
    );
    assert_eq!(err_bad_req.error_code(), "BAD_REQUEST");

    let err_domain = ApiError::Domain(domain_types::DomainError::MathOverflow);
    assert_eq!(
        err_domain.status_code(),
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(err_domain.error_code(), "MATH_OVERFLOW");
}

#[tokio::test]
async fn test_persistence_repositories_in_memory_fallback() {
    let db = DatabasePool::disabled();
    assert!(!db.is_connected());

    // 1. Snapshot repo
    let snap_repo = SnapshotRepository::new(db.clone());
    let pos =
        domain_types::PositionSnapshot::synthetic_demo("test-pos", "owner-1", 1000, 500, 8500);
    assert!(snap_repo.save_snapshot(&pos).await.is_ok());
    let loaded = snap_repo.get_snapshot("test-pos").await.unwrap().unwrap();
    assert_eq!(loaded.id, "test-pos");
    assert_eq!(loaded.owner, "owner-1");

    // 2. Observation repo
    let obs_repo = ObservationRepository::new(db.clone());
    let price = Price::new(150_00000000, -8, 100, 1700000000, 10);
    assert!(obs_repo
        .save_observation("SOL", "PythMock", &price)
        .await
        .is_ok());
    let loaded_p = obs_repo
        .get_latest_observation("SOL")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(loaded_p.mantissa, 150_00000000);

    // 3. Recommendation repo
    let rec_repo = RecommendationRepository::new(db.clone());
    let rec_id = Uuid::new_v4();
    let req_params = json!({"action": "RepayDebt"});
    let ranked = json!({"status": "Quoted", "plans": []});
    assert!(rec_repo
        .save_recommendation(rec_id, "test-pos", req_params, ranked, "Quoted", 300)
        .await
        .is_ok());

    let loaded_rec = rec_repo.get_recommendation(rec_id).await.unwrap().unwrap();
    assert_eq!(loaded_rec.recovery_id, rec_id);
    assert_eq!(loaded_rec.position_id, "test-pos");
    assert_eq!(loaded_rec.status, "Quoted");
}
