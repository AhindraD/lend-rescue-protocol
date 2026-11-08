use std::sync::Arc;

use crate::config::AppConfig;
use crate::persistence::{
    DatabasePool, ObservationRepository, RecommendationRepository, SnapshotRepository,
};
use crate::providers::{DemoProvider, OracleProvider, SwapQuoteProvider};
use crate::services::{MarketService, PositionService, RecoveryService, RiskService};

/// Global application shared state.
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub market_service: MarketService,
    pub position_service: PositionService,
    pub risk_service: RiskService,
    pub recovery_service: RecoveryService,
    pub oracle_provider: Arc<dyn OracleProvider>,
    pub swap_quote_provider: Arc<dyn SwapQuoteProvider>,
    pub db_pool: DatabasePool,
}

impl AppState {
    /// Creates AppState with deterministic demo provider and optional PostgreSQL pool.
    pub async fn create(config: AppConfig) -> Result<Self, crate::error::ApiError> {
        let db_pool = DatabasePool::init(config.database_url.as_deref(), config.demo_mode).await?;

        let demo_provider = Arc::new(DemoProvider::new());
        let oracle_provider: Arc<dyn OracleProvider> =
            match config.oracle_source.to_ascii_lowercase().as_str() {
                "mock" => Arc::new(crate::adapters::MockOracleProvider::new()),
                "pyth" | "surfpool" => {
                    let pyth_config = crate::adapters::PythCoreConfig::new(
                        config.solana_rpc_url.clone(),
                        config.solana_network.clone(),
                    )
                    .with_max_age_secs(config.pyth_max_age_secs)
                    .with_max_confidence_ratio_bps(config.pyth_max_confidence_ratio_bps)
                    .with_api_key(config.pyth_api_key.clone());

                    let rpc_transport = Arc::new(crate::adapters::HttpSolanaRpcTransport::new(
                        &config.solana_rpc_url,
                        config.pyth_request_timeout_ms,
                    ));
                    Arc::new(crate::adapters::PythOracleProvider::new(
                        pyth_config,
                        rpc_transport,
                        None,
                    ))
                }
                "pyth_mock" => {
                    let pyth_config = crate::adapters::PythCoreConfig::new(
                        config.solana_rpc_url.clone(),
                        config.solana_network.clone(),
                    )
                    .with_max_age_secs(config.pyth_max_age_secs)
                    .with_max_confidence_ratio_bps(config.pyth_max_confidence_ratio_bps)
                    .with_api_key(config.pyth_api_key.clone());

                    let rpc_transport = Arc::new(crate::adapters::MockSolanaRpcTransport::new());
                    Arc::new(crate::adapters::PythOracleProvider::new(
                        pyth_config,
                        rpc_transport,
                        None,
                    ))
                }
                _ => demo_provider.clone(),
            };

        let position_provider: Arc<dyn crate::providers::PositionProvider> =
            match config.position_source.to_ascii_lowercase().as_str() {
                "kamino" => {
                    let kamino_config = crate::adapters::KaminoAdapterConfig::new(
                        &config.kamino_adapter_url,
                        &config.kamino_market_address,
                    )
                    .with_request_timeout_ms(config.kamino_request_timeout_ms);

                    let transport = Arc::new(crate::adapters::KaminoHttpTransport::new(
                        kamino_config.clone(),
                    ));
                    Arc::new(crate::adapters::KaminoPositionProvider::new(
                        kamino_config,
                        transport,
                    ))
                }
                "mock" | "kamino_mock" => {
                    let kamino_config = crate::adapters::KaminoAdapterConfig::default();
                    let transport = Arc::new(crate::adapters::MockKaminoTransport::new());
                    Arc::new(crate::adapters::KaminoPositionProvider::new(
                        kamino_config,
                        transport,
                    ))
                }
                _ => demo_provider.clone(),
            };

        let swap_quote_provider: Arc<dyn SwapQuoteProvider> =
            match config.swap_quote_source.to_ascii_lowercase().as_str() {
                "jupiter" => {
                    let jup_config = crate::adapters::JupiterConfig::new(&config.jupiter_api_url)
                        .with_api_key(config.jupiter_api_key.clone())
                        .with_request_timeout_ms(config.jupiter_request_timeout_ms)
                        .with_max_retries(config.jupiter_max_retries)
                        .with_quote_ttl_secs(config.jupiter_quote_ttl_secs);

                    let transport = Arc::new(crate::adapters::JupiterHttpTransport::new(
                        jup_config.clone(),
                    ));
                    Arc::new(crate::adapters::JupiterSwapQuoteProvider::new(
                        jup_config, transport,
                    ))
                }
                _ => demo_provider.clone(),
            };

        let snapshot_repo = SnapshotRepository::new(db_pool.clone());
        let _observation_repo = ObservationRepository::new(db_pool.clone());
        let recommendation_repo = RecommendationRepository::new(db_pool.clone());

        let market_service = MarketService::new(demo_provider.clone());
        let position_service = PositionService::new(position_provider.clone(), snapshot_repo);
        let risk_service = RiskService::new(position_service.clone(), oracle_provider.clone());
        let recovery_service = RecoveryService::new(
            position_service.clone(),
            oracle_provider.clone(),
            swap_quote_provider.clone(),
            demo_provider.swap_pool_params(),
            recommendation_repo,
        );

        Ok(Self {
            config,
            market_service,
            position_service,
            risk_service,
            recovery_service,
            oracle_provider,
            swap_quote_provider,
            db_pool,
        })
    }
}
