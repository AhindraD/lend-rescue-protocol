use core::fmt;
use serde::{Deserialize, Serialize};

/// Standard asset identifier for supported and synthetic tokens.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AssetId {
    /// Native or Wrapped SOL
    Sol,
    /// USD Coin
    Usdc,
    /// Synthetic SOL (Devnet demo collateral)
    SSol,
    /// Synthetic USDC (Devnet demo debt)
    SUsdc,
    /// Custom asset identified by base58 mint string or symbol
    Custom(String),
}

impl AssetId {
    /// Default decimals for the asset.
    pub fn default_decimals(&self) -> u8 {
        match self {
            AssetId::Sol | AssetId::SSol => 9,
            AssetId::Usdc | AssetId::SUsdc => 6,
            AssetId::Custom(_) => 6,
        }
    }

    /// Symbol string.
    pub fn symbol(&self) -> &str {
        match self {
            AssetId::Sol => "SOL",
            AssetId::Usdc => "USDC",
            AssetId::SSol => "sSOL",
            AssetId::SUsdc => "sUSDC",
            AssetId::Custom(s) => s.as_str(),
        }
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

/// Metadata describing an asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    pub id: AssetId,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub mint: Option<String>,
}

impl Asset {
    pub fn new(id: AssetId, name: impl Into<String>, decimals: u8, mint: Option<String>) -> Self {
        let symbol = id.symbol().to_string();
        Self {
            id,
            symbol,
            name: name.into(),
            decimals,
            mint,
        }
    }

    pub fn s_sol(mint: Option<String>) -> Self {
        Self::new(AssetId::SSol, "Synthetic SOL", 9, mint)
    }

    pub fn s_usdc(mint: Option<String>) -> Self {
        Self::new(AssetId::SUsdc, "Synthetic USD Coin", 6, mint)
    }
}
