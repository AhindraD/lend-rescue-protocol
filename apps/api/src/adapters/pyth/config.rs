use std::collections::HashMap;

use domain_types::AssetId;

use super::account::PYTH_RECEIVER_PROGRAM_ID;

/// Configuration for the Pyth Core price provider.
#[derive(Clone)]
pub struct PythCoreConfig {
    pub rpc_url: String,
    pub network: String,
    pub program_id: String,
    pub feed_ids: HashMap<AssetId, [u8; 32]>,
    pub feed_accounts: HashMap<AssetId, String>,
    pub max_observation_age_secs: i64,
    pub max_confidence_ratio_bps: u64,
    pub request_timeout_ms: u64,
    pub api_key: Option<String>,
    pub hermes_url: Option<String>,
}

impl PythCoreConfig {
    /// Creates configuration with official default Pyth feed IDs and receiver program.
    pub fn new(rpc_url: impl Into<String>, network: impl Into<String>) -> Self {
        let mut feed_ids = HashMap::new();

        // Official Pyth Feed IDs
        // SOL/USD
        if let Ok(bytes) =
            hex::decode("ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d")
        {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            feed_ids.insert(AssetId::Sol, arr);
            feed_ids.insert(AssetId::SSol, arr);
        }

        // USDC/USD
        if let Ok(bytes) =
            hex::decode("eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a")
        {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            feed_ids.insert(AssetId::Usdc, arr);
            feed_ids.insert(AssetId::SUsdc, arr);
        }

        Self {
            rpc_url: rpc_url.into(),
            network: network.into(),
            program_id: PYTH_RECEIVER_PROGRAM_ID.to_string(),
            feed_ids,
            feed_accounts: HashMap::new(),
            max_observation_age_secs: 60,
            max_confidence_ratio_bps: 200, // 2.0%
            request_timeout_ms: 5000,
            api_key: None,
            hermes_url: Some("https://hermes.pyth.network".to_string()),
        }
    }

    /// Sets explicit account address for an asset.
    pub fn with_feed_account(
        mut self,
        asset_id: AssetId,
        account_pubkey: impl Into<String>,
    ) -> Self {
        self.feed_accounts.insert(asset_id, account_pubkey.into());
        self
    }

    /// Sets explicit feed ID for an asset.
    pub fn with_feed_id(mut self, asset_id: AssetId, feed_id: [u8; 32]) -> Self {
        self.feed_ids.insert(asset_id, feed_id);
        self
    }

    /// Sets maximum permitted age of an observation in seconds.
    pub fn with_max_age_secs(mut self, age: i64) -> Self {
        self.max_observation_age_secs = age;
        self
    }

    /// Sets maximum confidence ratio in basis points.
    pub fn with_max_confidence_ratio_bps(mut self, bps: u64) -> Self {
        self.max_confidence_ratio_bps = bps;
        self
    }

    /// Sets API key for authenticated Hermes requests.
    pub fn with_api_key(mut self, key: Option<String>) -> Self {
        self.api_key = key;
        self
    }
}

impl std::fmt::Debug for PythCoreConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PythCoreConfig")
            .field("rpc_url", &self.rpc_url)
            .field("network", &self.network)
            .field("program_id", &self.program_id)
            .field("feed_ids_count", &self.feed_ids.len())
            .field("feed_accounts_count", &self.feed_accounts.len())
            .field("max_observation_age_secs", &self.max_observation_age_secs)
            .field("max_confidence_ratio_bps", &self.max_confidence_ratio_bps)
            .field("request_timeout_ms", &self.request_timeout_ms)
            .field("api_key", &self.api_key.as_ref().map(|_| "[REDACTED]"))
            .field("hermes_url", &self.hermes_url)
            .finish()
    }
}
