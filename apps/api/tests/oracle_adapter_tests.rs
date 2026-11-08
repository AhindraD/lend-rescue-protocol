use chrono::Utc;
use domain_types::AssetId;
use lend_rescue_api::adapters::pyth::{
    AccountDataResult, MockHermesTransport, MockSolanaRpcTransport, PriceFeedMessage,
    PriceUpdateV2, PythCoreConfig, VerificationLevel, PYTH_RECEIVER_PROGRAM_ID,
};
use lend_rescue_api::adapters::{MockOracleProvider, PythOracleProvider};
use lend_rescue_api::providers::{OracleProvider, OracleSource};
use std::sync::Arc;

fn make_sample_price_message(
    feed_id: [u8; 32],
    price: i64,
    conf: u64,
    expo: i32,
    publish_time: i64,
) -> PriceFeedMessage {
    PriceFeedMessage {
        feed_id,
        price,
        conf,
        expo,
        publish_time,
        prev_publish_time: publish_time - 1,
        ema_price: price,
        ema_conf: conf,
    }
}

fn sol_feed_id() -> [u8; 32] {
    let mut arr = [0u8; 32];
    let bytes =
        hex::decode("ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d").unwrap();
    arr.copy_from_slice(&bytes);
    arr
}

fn btc_feed_id() -> [u8; 32] {
    let mut arr = [0u8; 32];
    let bytes =
        hex::decode("e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43").unwrap();
    arr.copy_from_slice(&bytes);
    arr
}

#[tokio::test]
async fn test_valid_pyth_observation_decoding() {
    let now = Utc::now().timestamp();
    let sol_feed = sol_feed_id();
    let account = PriceUpdateV2 {
        write_authority: [1u8; 32],
        verification_level: VerificationLevel::Full,
        price_message: make_sample_price_message(sol_feed, 150_00000000, 500000, -8, now),
        posted_slot: 250_000,
    };
    let account_bytes = account.to_account_bytes().unwrap();

    let rpc = Arc::new(MockSolanaRpcTransport::new());
    let sol_account_pubkey = "H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG";
    rpc.set_account(
        sol_account_pubkey,
        AccountDataResult {
            owner: PYTH_RECEIVER_PROGRAM_ID.to_string(),
            data: account_bytes,
            slot: 250_000,
        },
    )
    .await;

    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet")
        .with_feed_account(AssetId::Sol, sol_account_pubkey);

    let provider = PythOracleProvider::new(config, rpc, None);

    let obs = provider
        .get_observation(&AssetId::Sol)
        .await
        .unwrap()
        .expect("Observation must be found");

    assert_eq!(obs.price.mantissa, 150_00000000);
    assert_eq!(obs.price.exponent, -8);
    assert_eq!(obs.price.confidence, 500000);
    assert_eq!(obs.price.timestamp, now);
    assert_eq!(obs.slot, Some(250_000));
    assert_eq!(obs.source, OracleSource::MainnetObserved);
    assert_eq!(obs.source.as_str(), "Mainnet Observed");
    assert!(obs.is_valid);
}

#[tokio::test]
async fn test_correct_feed_id_verification() {
    let now = Utc::now().timestamp();
    let sol_feed = sol_feed_id();
    let account = PriceUpdateV2 {
        write_authority: [2u8; 32],
        verification_level: VerificationLevel::Full,
        price_message: make_sample_price_message(sol_feed, 155_00000000, 100000, -8, now),
        posted_slot: 300_000,
    };

    let rpc = Arc::new(MockSolanaRpcTransport::new());
    rpc.set_account(
        "sol_acc",
        AccountDataResult {
            owner: PYTH_RECEIVER_PROGRAM_ID.to_string(),
            data: account.to_account_bytes().unwrap(),
            slot: 300_000,
        },
    )
    .await;

    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet")
        .with_feed_account(AssetId::Sol, "sol_acc");
    let provider = PythOracleProvider::new(config, rpc, None);

    let price = provider.get_price(&AssetId::Sol).await.unwrap().unwrap();
    assert_eq!(price.mantissa, 155_00000000);
}

#[tokio::test]
async fn test_incorrect_feed_id_rejection() {
    let now = Utc::now().timestamp();
    // Intentionally put BTC feed ID into the account mapped for SOL
    let btc_feed = btc_feed_id();
    let account = PriceUpdateV2 {
        write_authority: [3u8; 32],
        verification_level: VerificationLevel::Full,
        price_message: make_sample_price_message(btc_feed, 60_000 * 100_000_000, 5000000, -8, now),
        posted_slot: 300_000,
    };

    let rpc = Arc::new(MockSolanaRpcTransport::new());
    rpc.set_account(
        "sol_acc",
        AccountDataResult {
            owner: PYTH_RECEIVER_PROGRAM_ID.to_string(),
            data: account.to_account_bytes().unwrap(),
            slot: 300_000,
        },
    )
    .await;

    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet")
        .with_feed_account(AssetId::Sol, "sol_acc");
    let provider = PythOracleProvider::new(config, rpc, None);

    let err = provider.get_price(&AssetId::Sol).await.unwrap_err();
    assert!(err.to_string().contains("Feed ID mismatch"));
}

#[tokio::test]
async fn test_invalid_account_owner_rejection() {
    let now = Utc::now().timestamp();
    let sol_feed = sol_feed_id();
    let account = PriceUpdateV2 {
        write_authority: [4u8; 32],
        verification_level: VerificationLevel::Full,
        price_message: make_sample_price_message(sol_feed, 150_00000000, 500000, -8, now),
        posted_slot: 300_000,
    };

    let rpc = Arc::new(MockSolanaRpcTransport::new());
    // Rogue account owner (not PYTH_RECEIVER_PROGRAM_ID)
    rpc.set_account(
        "sol_acc",
        AccountDataResult {
            owner: "11111111111111111111111111111111".to_string(),
            data: account.to_account_bytes().unwrap(),
            slot: 300_000,
        },
    )
    .await;

    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet")
        .with_feed_account(AssetId::Sol, "sol_acc");
    let provider = PythOracleProvider::new(config, rpc, None);

    let err = provider.get_price(&AssetId::Sol).await.unwrap_err();
    assert!(err.to_string().contains("Invalid account owner"));
}

#[tokio::test]
async fn test_invalid_account_data_rejection() {
    let rpc = Arc::new(MockSolanaRpcTransport::new());
    // Corrupted binary payload with invalid discriminator
    let corrupted_data = vec![0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33, 0x44];
    rpc.set_account(
        "sol_acc",
        AccountDataResult {
            owner: PYTH_RECEIVER_PROGRAM_ID.to_string(),
            data: corrupted_data,
            slot: 300_000,
        },
    )
    .await;

    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet")
        .with_feed_account(AssetId::Sol, "sol_acc");
    let provider = PythOracleProvider::new(config, rpc, None);

    let err = provider.get_price(&AssetId::Sol).await.unwrap_err();
    assert!(
        err.to_string().contains("Invalid account data")
            || err.to_string().contains("discriminator")
    );
}

#[tokio::test]
async fn test_non_positive_price_rejection() {
    let now = Utc::now().timestamp();
    let sol_feed = sol_feed_id();
    // Zero price
    let account = PriceUpdateV2 {
        write_authority: [5u8; 32],
        verification_level: VerificationLevel::Full,
        price_message: make_sample_price_message(sol_feed, 0, 500000, -8, now),
        posted_slot: 300_000,
    };

    let rpc = Arc::new(MockSolanaRpcTransport::new());
    rpc.set_account(
        "sol_acc",
        AccountDataResult {
            owner: PYTH_RECEIVER_PROGRAM_ID.to_string(),
            data: account.to_account_bytes().unwrap(),
            slot: 300_000,
        },
    )
    .await;

    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet")
        .with_feed_account(AssetId::Sol, "sol_acc");
    let provider = PythOracleProvider::new(config, rpc, None);

    let err = provider.get_price(&AssetId::Sol).await.unwrap_err();
    assert!(err.to_string().contains("Non-positive price"));
}

#[tokio::test]
async fn test_excessive_confidence_rejection() {
    let now = Utc::now().timestamp();
    let sol_feed = sol_feed_id();
    // Price = $100 (100_00000000). Max conf = 200 bps (2%) = 2_00000000.
    // Set conf = 5_00000000 (5%), which exceeds 200 bps!
    let account = PriceUpdateV2 {
        write_authority: [6u8; 32],
        verification_level: VerificationLevel::Full,
        price_message: make_sample_price_message(sol_feed, 100_00000000, 5_00000000, -8, now),
        posted_slot: 300_000,
    };

    let rpc = Arc::new(MockSolanaRpcTransport::new());
    rpc.set_account(
        "sol_acc",
        AccountDataResult {
            owner: PYTH_RECEIVER_PROGRAM_ID.to_string(),
            data: account.to_account_bytes().unwrap(),
            slot: 300_000,
        },
    )
    .await;

    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet")
        .with_feed_account(AssetId::Sol, "sol_acc")
        .with_max_confidence_ratio_bps(200);
    let provider = PythOracleProvider::new(config, rpc, None);

    let err = provider.get_price(&AssetId::Sol).await.unwrap_err();
    assert!(err.to_string().contains("Confidence interval too wide"));
}

#[tokio::test]
async fn test_stale_observation_rejection() {
    let now = Utc::now().timestamp();
    let sol_feed = sol_feed_id();
    // Observation is 120 seconds old; max permitted is 60s
    let stale_time = now - 120;
    let account = PriceUpdateV2 {
        write_authority: [7u8; 32],
        verification_level: VerificationLevel::Full,
        price_message: make_sample_price_message(sol_feed, 150_00000000, 500000, -8, stale_time),
        posted_slot: 300_000,
    };

    let rpc = Arc::new(MockSolanaRpcTransport::new());
    rpc.set_account(
        "sol_acc",
        AccountDataResult {
            owner: PYTH_RECEIVER_PROGRAM_ID.to_string(),
            data: account.to_account_bytes().unwrap(),
            slot: 300_000,
        },
    )
    .await;

    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet")
        .with_feed_account(AssetId::Sol, "sol_acc")
        .with_max_age_secs(60);
    let provider = PythOracleProvider::new(config, rpc, None);

    let err = provider.get_price(&AssetId::Sol).await.unwrap_err();
    assert!(err.to_string().contains("Price observation is stale"));
}

#[tokio::test]
async fn test_future_publication_time_rejection() {
    let now = Utc::now().timestamp();
    let sol_feed = sol_feed_id();
    // Observation is 100 seconds into the future (impossible timestamp)
    let future_time = now + 100;
    let account = PriceUpdateV2 {
        write_authority: [8u8; 32],
        verification_level: VerificationLevel::Full,
        price_message: make_sample_price_message(sol_feed, 150_00000000, 500000, -8, future_time),
        posted_slot: 300_000,
    };

    let rpc = Arc::new(MockSolanaRpcTransport::new());
    rpc.set_account(
        "sol_acc",
        AccountDataResult {
            owner: PYTH_RECEIVER_PROGRAM_ID.to_string(),
            data: account.to_account_bytes().unwrap(),
            slot: 300_000,
        },
    )
    .await;

    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet")
        .with_feed_account(AssetId::Sol, "sol_acc");
    let provider = PythOracleProvider::new(config, rpc, None);

    let err = provider.get_price(&AssetId::Sol).await.unwrap_err();
    assert!(err.to_string().contains("future"));
}

#[tokio::test]
async fn test_invalid_exponent_rejection() {
    let now = Utc::now().timestamp();
    let sol_feed = sol_feed_id();
    // Unsupported positive exponent +2
    let account = PriceUpdateV2 {
        write_authority: [9u8; 32],
        verification_level: VerificationLevel::Full,
        price_message: make_sample_price_message(sol_feed, 150_00000000, 500000, 2, now),
        posted_slot: 300_000,
    };

    let rpc = Arc::new(MockSolanaRpcTransport::new());
    rpc.set_account(
        "sol_acc",
        AccountDataResult {
            owner: PYTH_RECEIVER_PROGRAM_ID.to_string(),
            data: account.to_account_bytes().unwrap(),
            slot: 300_000,
        },
    )
    .await;

    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet")
        .with_feed_account(AssetId::Sol, "sol_acc");
    let provider = PythOracleProvider::new(config, rpc, None);

    let err = provider.get_price(&AssetId::Sol).await.unwrap_err();
    assert!(err.to_string().contains("Invalid or unsupported exponent"));
}

#[tokio::test]
async fn test_rpc_timeout_handling() {
    let rpc = Arc::new(MockSolanaRpcTransport::new());
    rpc.set_timeout(true);

    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet")
        .with_feed_account(AssetId::Sol, "sol_acc");
    let provider = PythOracleProvider::new(config, rpc, None);

    let err = provider.get_price(&AssetId::Sol).await.unwrap_err();
    assert!(err.to_string().contains("timed out"));
}

#[tokio::test]
async fn test_missing_feed_mapping() {
    let rpc = Arc::new(MockSolanaRpcTransport::new());
    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet");
    let provider = PythOracleProvider::new(config, rpc, None);

    let err = provider
        .get_price(&AssetId::Custom("UNKNOWN_COIN".to_string()))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("No Pyth feed ID configured"));
}

#[tokio::test]
async fn test_hermes_authentication_failure() {
    let hermes = Arc::new(MockHermesTransport::new());
    hermes.set_require_auth(true);
    hermes
        .set_valid_api_key(Some("secret-key".to_string()))
        .await;
    hermes
        .set_caller_api_key(Some("wrong-key".to_string()))
        .await;

    let rpc = Arc::new(MockSolanaRpcTransport::new());
    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet")
        .with_api_key(Some("wrong-key".to_string()));

    let provider = PythOracleProvider::new(config, rpc, Some(hermes));

    let err = provider.get_price(&AssetId::Sol).await.unwrap_err();
    assert!(
        err.to_string().contains("Unauthorized")
            || err.to_string().contains("Invalid or missing Pyth API key")
    );
}

#[tokio::test]
async fn test_mock_observation_separate_from_pyth() {
    let mock_provider = MockOracleProvider::new();
    let obs = mock_provider
        .get_observation(&AssetId::Sol)
        .await
        .unwrap()
        .expect("Mock observation must be present");

    assert_eq!(obs.price.mantissa, 150_00000000);
    assert_eq!(obs.source, OracleSource::SyntheticFixture);
    assert_ne!(obs.source, OracleSource::MainnetObserved);
    assert_eq!(obs.source.as_str(), "Synthetic / Test Fixture");
}

#[tokio::test]
async fn test_no_silent_mock_fallback_on_pyth_failure() {
    let rpc = Arc::new(MockSolanaRpcTransport::new());
    rpc.set_timeout(true);

    let config = PythCoreConfig::new("http://127.0.0.1:8899", "mainnet")
        .with_feed_account(AssetId::Sol, "sol_acc");
    let provider = PythOracleProvider::new(config, rpc, None);

    // When Pyth fails, it MUST return an Err, NOT silently fall back to mock data
    let result = provider.get_price(&AssetId::Sol).await;
    assert!(
        result.is_err(),
        "Must return error when Pyth fails; no silent mock fallback!"
    );
}

#[tokio::test]
async fn test_correct_provenance_labels() {
    assert_eq!(OracleSource::MainnetObserved.as_str(), "Mainnet Observed");
    assert_eq!(OracleSource::DevnetExecuted.as_str(), "Devnet Executed");
    assert_eq!(
        OracleSource::SurfpoolSimulated.as_str(),
        "Surfpool Simulated"
    );
    assert_eq!(
        OracleSource::SyntheticFixture.as_str(),
        "Synthetic / Test Fixture"
    );

    let surfpool_config = PythCoreConfig::new("http://127.0.0.1:8899", "surfpool");
    let rpc = Arc::new(MockSolanaRpcTransport::new());
    let provider = PythOracleProvider::new(surfpool_config, rpc, None);
    assert_eq!(
        provider
            .with_source(OracleSource::SurfpoolSimulated)
            .config()
            .network,
        "surfpool"
    );
}

#[tokio::test]
async fn test_legacy_pyth_account_decoding() {
    let now = Utc::now().timestamp();
    // Build legacy Pyth account bytes
    let mut data = vec![0u8; 256];
    // magic = 0xa1b2c3d4
    data[0..4].copy_from_slice(&0xa1b2c3d4u32.to_le_bytes());
    // expo = -8 at offset 20
    data[20..24].copy_from_slice(&(-8i32).to_le_bytes());
    // price = 142_50000000 at offset 208
    data[208..216].copy_from_slice(&142_50000000i64.to_le_bytes());
    // conf = 200000 at offset 216
    data[216..224].copy_from_slice(&200000u64.to_le_bytes());
    // pub_slot = 190_000 at offset 232
    data[232..240].copy_from_slice(&190_000u64.to_le_bytes());
    // timestamp = now at offset 240
    data[240..248].copy_from_slice(&now.to_le_bytes());

    let decoded = lend_rescue_api::adapters::pyth::try_decode_pyth_account(&data).unwrap();
    assert!(decoded.is_legacy);
    assert_eq!(decoded.price, 142_50000000);
    assert_eq!(decoded.conf, 200000);
    assert_eq!(decoded.expo, -8);
    assert_eq!(decoded.slot, 190_000);
    assert_eq!(decoded.publish_time, now);
}
