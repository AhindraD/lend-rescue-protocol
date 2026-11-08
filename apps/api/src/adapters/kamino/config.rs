/// Configuration for internal Kamino adapter HTTP micro-service.
#[derive(Debug, Clone)]
pub struct KaminoAdapterConfig {
    /// Internal HTTP endpoint of the isolated Kamino TypeScript adapter
    pub adapter_url: String,
    /// Default Kamino market address to query if unspecified
    pub default_market: String,
    /// Timeout in milliseconds for adapter HTTP calls
    pub request_timeout_ms: u64,
    /// Maximum allowable slot drift
    pub max_slot_drift: u64,
}

impl Default for KaminoAdapterConfig {
    fn default() -> Self {
        Self {
            adapter_url: "http://127.0.0.1:3002".to_string(),
            default_market: "7u3HeHxatcAACvdFtHGryaZWwZxKhqWisdHJaJBYbugj".to_string(),
            request_timeout_ms: 10_000,
            max_slot_drift: 50,
        }
    }
}

impl KaminoAdapterConfig {
    pub fn new(adapter_url: impl Into<String>, default_market: impl Into<String>) -> Self {
        Self {
            adapter_url: adapter_url.into(),
            default_market: default_market.into(),
            request_timeout_ms: 10_000,
            max_slot_drift: 50,
        }
    }

    pub fn with_request_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.request_timeout_ms = timeout_ms;
        self
    }

    pub fn with_max_slot_drift(mut self, max_slot_drift: u64) -> Self {
        self.max_slot_drift = max_slot_drift;
        self
    }
}
