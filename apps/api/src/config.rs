use std::env;
use std::net::IpAddr;

/// Service configuration loaded from environment variables with safe defaults.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_host: IpAddr,
    pub bind_port: u16,
    pub database_url: Option<String>,
    pub cors_allowed_origins: Vec<String>,
    pub log_level: String,
    pub request_timeout_secs: u64,
    pub demo_mode: bool,
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

        Ok(Self {
            bind_host,
            bind_port,
            database_url,
            cors_allowed_origins,
            log_level,
            request_timeout_secs,
            demo_mode,
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
