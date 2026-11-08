use std::env;
use std::fmt;
use std::net::IpAddr;

/// Service configuration loaded from environment variables with safe defaults.
#[derive(Clone)]
pub struct AppConfig {
    pub bind_host: IpAddr,
    pub bind_port: u16,
    pub database_url: Option<String>,
    pub cors_allowed_origins: Vec<String>,
    pub log_level: String,
    pub request_timeout_secs: u64,
    pub demo_mode: bool,

    // Oracle & Protocol Configuration
    pub solana_rpc_url: String,
    pub solana_network: String,
    pub pyth_program_id: String,
    pub pyth_max_age_secs: i64,
    pub pyth_max_confidence_ratio_bps: u64,
    pub pyth_request_timeout_ms: u64,
    pub pyth_api_key: Option<String>,
    pub hermes_url: Option<String>,
    pub oracle_source: String,

    // Kamino Protocol Adapter Configuration (Phase 5B)
    pub kamino_adapter_url: String,
    pub kamino_market_address: String,
    pub kamino_request_timeout_ms: u64,
    pub position_source: String,

    // Jupiter Swap Adapter Configuration (Phase 5C)
    pub jupiter_api_url: String,
    pub jupiter_api_key: Option<String>,
    pub jupiter_request_timeout_ms: u64,
    pub jupiter_max_retries: usize,
    pub jupiter_quote_ttl_secs: i64,
    pub swap_quote_source: String,
}

impl fmt::Debug for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppConfig")
            .field("bind_host", &self.bind_host)
            .field("bind_port", &self.bind_port)
            .field("database_url", &self.database_url)
            .field("cors_allowed_origins", &self.cors_allowed_origins)
            .field("log_level", &self.log_level)
            .field("request_timeout_secs", &self.request_timeout_secs)
            .field("demo_mode", &self.demo_mode)
            .field("solana_rpc_url", &self.solana_rpc_url)
            .field("solana_network", &self.solana_network)
            .field("pyth_program_id", &self.pyth_program_id)
            .field("pyth_max_age_secs", &self.pyth_max_age_secs)
            .field(
                "pyth_max_confidence_ratio_bps",
                &self.pyth_max_confidence_ratio_bps,
            )
            .field("pyth_request_timeout_ms", &self.pyth_request_timeout_ms)
            .field(
                "pyth_api_key",
                &self.pyth_api_key.as_ref().map(|_| "[REDACTED]"),
            )
            .field("hermes_url", &self.hermes_url)
            .field("oracle_source", &self.oracle_source)
            .field("kamino_adapter_url", &self.kamino_adapter_url)
            .field("kamino_market_address", &self.kamino_market_address)
            .field("kamino_request_timeout_ms", &self.kamino_request_timeout_ms)
            .field("position_source", &self.position_source)
            .field("jupiter_api_url", &self.jupiter_api_url)
            .field(
                "jupiter_api_key",
                &self.jupiter_api_key.as_ref().map(|_| "[REDACTED]"),
            )
            .field(
                "jupiter_request_timeout_ms",
                &self.jupiter_request_timeout_ms,
            )
            .field("jupiter_max_retries", &self.jupiter_max_retries)
            .field("jupiter_quote_ttl_secs", &self.jupiter_quote_ttl_secs)
            .field("swap_quote_source", &self.swap_quote_source)
            .finish()
    }
}

impl AppConfig {
    /// Loads configuration from environment variables.
    pub fn from_env() -> Result<Self, String> {
        let bind_host: IpAddr = env::var("BIND_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string())
            .parse()
            .map_err(|e| format!("Invalid BIND_HOST: {}", e))?;

        let bind_port: u16 = env::var("BIND_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|e| format!("Invalid BIND_PORT: {}", e))?;

        let database_url = env::var("DATABASE_URL")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://127.0.0.1:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        let request_timeout_secs: u64 = env::var("REQUEST_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .map_err(|e| format!("Invalid REQUEST_TIMEOUT_SECS: {}", e))?;

        let demo_mode = env::var("DEMO_MODE")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(true);

        let solana_rpc_url =
            env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8899".to_string());

        let solana_network = env::var("SOLANA_NETWORK").unwrap_or_else(|_| "surfpool".to_string());

        let pyth_program_id = env::var("PYTH_PROGRAM_ID")
            .unwrap_or_else(|_| "rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ".to_string());

        let pyth_max_age_secs: i64 = env::var("PYTH_MAX_AGE_SECS")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .map_err(|e| format!("Invalid PYTH_MAX_AGE_SECS: {}", e))?;

        let pyth_max_confidence_ratio_bps: u64 = env::var("PYTH_MAX_CONFIDENCE_RATIO_BPS")
            .unwrap_or_else(|_| "200".to_string())
            .parse()
            .map_err(|e| format!("Invalid PYTH_MAX_CONFIDENCE_RATIO_BPS: {}", e))?;

        let pyth_request_timeout_ms: u64 = env::var("PYTH_REQUEST_TIMEOUT_MS")
            .unwrap_or_else(|_| "5000".to_string())
            .parse()
            .map_err(|e| format!("Invalid PYTH_REQUEST_TIMEOUT_MS: {}", e))?;

        let pyth_api_key = env::var("PYTH_API_KEY")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let hermes_url = env::var("HERMES_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| Some("https://hermes.pyth.network".to_string()));

        let oracle_source = env::var("ORACLE_SOURCE").unwrap_or_else(|_| "demo".to_string());

        let kamino_adapter_url =
            env::var("KAMINO_ADAPTER_URL").unwrap_or_else(|_| "http://127.0.0.1:3002".to_string());

        let kamino_market_address = env::var("KAMINO_DEFAULT_MARKET")
            .unwrap_or_else(|_| "7u3HeHxatcAACvdFtHGryaZWwZxKhqWisdHJaJBYbugj".to_string());

        let kamino_request_timeout_ms: u64 = env::var("KAMINO_REQUEST_TIMEOUT_MS")
            .unwrap_or_else(|_| "10000".to_string())
            .parse()
            .map_err(|e| format!("Invalid KAMINO_REQUEST_TIMEOUT_MS: {}", e))?;

        let position_source = env::var("POSITION_SOURCE").unwrap_or_else(|_| "demo".to_string());

        let jupiter_api_url = env::var("JUPITER_API_URL")
            .unwrap_or_else(|_| "https://api.jup.ag/swap/v2".to_string());

        let jupiter_api_key = env::var("JUPITER_API_KEY")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let jupiter_request_timeout_ms: u64 = env::var("JUPITER_REQUEST_TIMEOUT_MS")
            .unwrap_or_else(|_| "5000".to_string())
            .parse()
            .map_err(|e| format!("Invalid JUPITER_REQUEST_TIMEOUT_MS: {}", e))?;

        let jupiter_max_retries: usize = env::var("JUPITER_MAX_RETRIES")
            .unwrap_or_else(|_| "2".to_string())
            .parse()
            .map_err(|e| format!("Invalid JUPITER_MAX_RETRIES: {}", e))?;

        let jupiter_quote_ttl_secs: i64 = env::var("JUPITER_QUOTE_TTL_SECS")
            .unwrap_or_else(|_| "15".to_string())
            .parse()
            .map_err(|e| format!("Invalid JUPITER_QUOTE_TTL_SECS: {}", e))?;

        let swap_quote_source =
            env::var("SWAP_QUOTE_SOURCE").unwrap_or_else(|_| "demo".to_string());

        Ok(Self {
            bind_host,
            bind_port,
            database_url,
            cors_allowed_origins,
            log_level,
            request_timeout_secs,
            demo_mode,
            solana_rpc_url,
            solana_network,
            pyth_program_id,
            pyth_max_age_secs,
            pyth_max_confidence_ratio_bps,
            pyth_request_timeout_ms,
            pyth_api_key,
            hermes_url,
            oracle_source,
            kamino_adapter_url,
            kamino_market_address,
            kamino_request_timeout_ms,
            position_source,
            jupiter_api_url,
            jupiter_api_key,
            jupiter_request_timeout_ms,
            jupiter_max_retries,
            jupiter_quote_ttl_secs,
            swap_quote_source,
        })
    }

    /// Default test configuration.
    pub fn for_test() -> Self {
        Self {
            bind_host: "127.0.0.1".parse().unwrap(),
            bind_port: 0,
            database_url: None,
            cors_allowed_origins: vec!["http://localhost:3000".to_string()],
            log_level: "info".to_string(),
            request_timeout_secs: 5,
            demo_mode: true,
            solana_rpc_url: "http://127.0.0.1:8899".to_string(),
            solana_network: "surfpool".to_string(),
            pyth_program_id: "rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ".to_string(),
            pyth_max_age_secs: 60,
            pyth_max_confidence_ratio_bps: 200,
            pyth_request_timeout_ms: 5000,
            pyth_api_key: None,
            hermes_url: Some("https://hermes.pyth.network".to_string()),
            oracle_source: "demo".to_string(),
            kamino_adapter_url: "http://127.0.0.1:3002".to_string(),
            kamino_market_address: "7u3HeHxatcAACvdFtHGryaZWwZxKhqWisdHJaJBYbugj".to_string(),
            kamino_request_timeout_ms: 10000,
            position_source: "demo".to_string(),
            jupiter_api_url: "https://api.jup.ag/swap/v2".to_string(),
            jupiter_api_key: None,
            jupiter_request_timeout_ms: 5000,
            jupiter_max_retries: 2,
            jupiter_quote_ttl_secs: 15,
            swap_quote_source: "demo".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_config() {
        let config = AppConfig::for_test();
        assert_eq!(config.bind_host.to_string(), "127.0.0.1");
        assert!(config.demo_mode);
    }
}
