use std::sync::Arc;

use crate::config::AppConfig;
use crate::persistence::{
    DatabasePool, ObservationRepository, RecommendationRepository, SnapshotRepository,
};
use crate::providers::{DemoProvider, OracleProvider};
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
                "pyth" => {
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

        let snapshot_repo = SnapshotRepository::new(db_pool.clone());
        let _observation_repo = ObservationRepository::new(db_pool.clone());
        let recommendation_repo = RecommendationRepository::new(db_pool.clone());

        let market_service = MarketService::new(demo_provider.clone());
        let position_service = PositionService::new(demo_provider.clone(), snapshot_repo);
        let risk_service = RiskService::new(position_service.clone(), oracle_provider.clone());
        let recovery_service = RecoveryService::new(
            position_service.clone(),
            oracle_provider.clone(),
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
            db_pool,
        })
    }
}
