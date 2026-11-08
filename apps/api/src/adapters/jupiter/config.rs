use std::fmt;

/// Configuration for the Jupiter Swap API V2 adapter.
#[derive(Clone)]
pub struct JupiterConfig {
    /// Base URL for Jupiter V2 API (default: https://api.jup.ag/swap/v2)
    pub api_url: String,
    /// Optional Jupiter API key for higher rate limits (never logged or exposed in DTOs)
    pub api_key: Option<String>,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Maximum number of retries for transient HTTP errors
    pub max_retries: usize,
    /// Internal quote validity time-to-live in seconds
    pub quote_ttl_secs: i64,
}

impl fmt::Debug for JupiterConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JupiterConfig")
            .field("api_url", &self.api_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "[REDACTED]"))
            .field("request_timeout_ms", &self.request_timeout_ms)
            .field("max_retries", &self.max_retries)
            .field("quote_ttl_secs", &self.quote_ttl_secs)
            .finish()
    }
}

impl JupiterConfig {
    pub fn new(api_url: impl Into<String>) -> Self {
        Self {
            api_url: api_url.into(),
            api_key: None,
            request_timeout_ms: 5000,
            max_retries: 2,
            quote_ttl_secs: 15,
        }
    }

    pub fn default_v2() -> Self {
        Self::new("https://api.jup.ag/swap/v2")
    }

    pub fn with_api_key(mut self, key: Option<String>) -> Self {
        self.api_key = key.filter(|k| !k.trim().is_empty());
        self
    }

    pub fn with_request_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.request_timeout_ms = timeout_ms;
        self
    }

    pub fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn with_quote_ttl_secs(mut self, ttl_secs: i64) -> Self {
        self.quote_ttl_secs = ttl_secs;
        self
    }
}

