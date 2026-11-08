use chrono::Utc;
use domain_types::{AssetId, Price};
use std::sync::Arc;

use super::account::try_decode_pyth_account;
use super::config::PythCoreConfig;
use super::error::PythError;
use super::transport::{HermesTransport, SolanaRpcTransport};
use super::validation::{
    validate_account_owner, validate_confidence, validate_exponent, validate_feed_id,
    validate_freshness, validate_price_positivity, validate_publication_time,
};
use crate::error::ApiError;
use crate::providers::oracle_provider::{
    BoxFuture, OracleObservation, OracleProvider, OracleSource,
};

/// Production-oriented read-only Pyth Core price provider.
pub struct PythOracleProvider {
    config: PythCoreConfig,
    rpc_transport: Arc<dyn SolanaRpcTransport>,
    hermes_transport: Option<Arc<dyn HermesTransport>>,
    source: OracleSource,
}

impl PythOracleProvider {
    pub fn new(
        config: PythCoreConfig,
        rpc_transport: Arc<dyn SolanaRpcTransport>,
        hermes_transport: Option<Arc<dyn HermesTransport>>,
    ) -> Self {
        let source = if config.network.eq_ignore_ascii_case("surfpool") {
            OracleSource::SurfpoolSimulated
        } else {
            OracleSource::MainnetObserved
        };

        Self {
            config,
            rpc_transport,
            hermes_transport,
            source,
        }
    }

    pub fn with_source(mut self, source: OracleSource) -> Self {
        self.source = source;
        self
    }

    pub fn config(&self) -> &PythCoreConfig {
        &self.config
    }
}

impl OracleProvider for PythOracleProvider {
    fn get_price<'a>(
        &'a self,
        asset_id: &'a AssetId,
    ) -> BoxFuture<'a, Result<Option<Price>, ApiError>> {
        Box::pin(async move {
            let obs_opt = self.get_observation(asset_id).await?;
            Ok(obs_opt.map(|obs| obs.price))
        })
    }

    fn get_observation<'a>(
        &'a self,
        asset_id: &'a AssetId,
    ) -> BoxFuture<'a, Result<Option<OracleObservation>, ApiError>> {
        Box::pin(async move {
            let now = Utc::now().timestamp();

            // 1. Resolve feed ID mapping for asset
            let expected_feed_id = self.config.feed_ids.get(asset_id).ok_or_else(|| {
                ApiError::NotFound(format!(
                    "No Pyth feed ID configured for asset {:?}",
                    asset_id
                ))
            })?;

            let feed_id_hex = hex::encode(expected_feed_id);

            // 2. Resolve on-chain account address (explicitly configured feed account)
            let account_address = self.config.feed_accounts.get(asset_id);

            if let Some(pubkey) = account_address {
                // 3. Query account from Solana RPC
                let account_opt = self
                    .rpc_transport
                    .get_account_data(pubkey)
                    .await
                    .map_err(ApiError::from)?;

                if let Some(account) = account_opt {
                    // 4. Validate account owner
                    validate_account_owner(&self.config.program_id, &account.owner)
                        .map_err(ApiError::from)?;

                    // 5. Decode account data (PriceUpdateV2 or Legacy)
                    let decoded = try_decode_pyth_account(&account.data).map_err(ApiError::from)?;

                    // 6. Validate feed ID if present in decoded payload
                    if let Some(actual_feed) = decoded.feed_id {
                        validate_feed_id(expected_feed_id, &actual_feed).map_err(ApiError::from)?;
                    }

                    // 7. Validate price positivity
                    validate_price_positivity(decoded.price).map_err(ApiError::from)?;

                    // 8. Validate confidence interval
                    validate_confidence(
                        decoded.price,
                        decoded.conf,
                        self.config.max_confidence_ratio_bps,
                    )
                    .map_err(ApiError::from)?;

                    // 9. Validate freshness
                    validate_freshness(
                        decoded.publish_time,
                        now,
                        self.config.max_observation_age_secs,
                    )
                    .map_err(ApiError::from)?;

                    // 10. Validate plausible publication time
                    validate_publication_time(decoded.publish_time, now, 10)
                        .map_err(ApiError::from)?;

                    // 11. Validate exponent
                    validate_exponent(decoded.expo).map_err(ApiError::from)?;

                    // 12. Construct normalized domain Price
                    let price = Price::new(
                        decoded.price,
                        decoded.expo,
                        decoded.conf,
                        decoded.publish_time,
                        decoded.slot,
                    )
                    .with_asset(asset_id.clone());

                    let age_secs = now - decoded.publish_time;

                    return Ok(Some(OracleObservation {
                        price,
                        feed_id: Some(feed_id_hex),
                        source: self.source,
                        is_valid: true,
                        age_secs,
                        slot: Some(decoded.slot),
                    }));
                }
            }

            // Fallback: If Hermes transport is available and configured
            if let Some(hermes) = &self.hermes_transport {
                let hermes_update = hermes
                    .get_latest_price_update(&feed_id_hex)
                    .await
                    .map_err(ApiError::from)?;

                // Validate feed ID
                if !hermes_update.feed_id.eq_ignore_ascii_case(&feed_id_hex) {
                    return Err(PythError::FeedIdMismatch {
                        expected: feed_id_hex,
                        actual: hermes_update.feed_id,
                    }
                    .into());
                }

                // Validate price positivity
                validate_price_positivity(hermes_update.price).map_err(ApiError::from)?;

                // Validate confidence
                validate_confidence(
                    hermes_update.price,
                    hermes_update.conf,
                    self.config.max_confidence_ratio_bps,
                )
                .map_err(ApiError::from)?;

                // Validate freshness
                validate_freshness(
                    hermes_update.publish_time,
                    now,
                    self.config.max_observation_age_secs,
                )
                .map_err(ApiError::from)?;

                // Validate publication time
                validate_publication_time(hermes_update.publish_time, now, 10)
                    .map_err(ApiError::from)?;

                // Validate exponent
                validate_exponent(hermes_update.expo).map_err(ApiError::from)?;

                let price = Price::new(
                    hermes_update.price,
                    hermes_update.expo,
                    hermes_update.conf,
                    hermes_update.publish_time,
                    hermes_update.slot.unwrap_or(0),
                )
                .with_asset(asset_id.clone());

                let age_secs = now - hermes_update.publish_time;

                return Ok(Some(OracleObservation {
                    price,
                    feed_id: Some(feed_id_hex),
                    source: self.source,
                    is_valid: true,
                    age_secs,
                    slot: hermes_update.slot,
                }));
            }

            Ok(None)
        })
    }
}
