use crate::constants::BPS_DIVISOR;
use anchor_lang::prelude::*;

#[account]
pub struct MockOracle {
    /// Market reference
    pub market: Pubkey,
    /// Asset mint this price feed reports
    pub asset_mint: Pubkey,
    /// Price mantissa (signed to support standard Pyth convention, but must be > 0)
    pub price: i64,
    /// Exponent (e.g. -6 means price * 10^-6)
    pub exponent: i32,
    /// Confidence interval in price base units
    pub confidence: u64,
    /// Slot of last price update
    pub last_update_slot: u64,
    /// Unix timestamp of last price update
    pub last_update_timestamp: i64,
    /// Authorized key that can update this oracle price
    pub update_authority: Pubkey,
    /// Schema version
    pub version: u8,
    /// PDA bump
    pub bump: u8,
}

impl MockOracle {
    pub const LEN: usize = 8 // discriminator
        + 32 // market
        + 32 // asset_mint
        + 8  // price
        + 4  // exponent
        + 8  // confidence
        + 8  // last_update_slot
        + 8  // last_update_timestamp
        + 32 // update_authority
        + 1  // version
        + 1  // bump
        + 32; // reserved padding

    /// Checks if the oracle price was updated within the permitted age threshold
    pub fn is_fresh(&self, current_timestamp: i64, max_age_secs: i64) -> bool {
        if self.last_update_timestamp > current_timestamp {
            // Future timestamps beyond reasonable clock skew indicate malformed state
            return false;
        }
        current_timestamp.saturating_sub(self.last_update_timestamp) <= max_age_secs
    }

    /// Checks if the confidence interval is within the acceptable proportion of price
    pub fn is_confidence_valid(&self, max_confidence_bps: u64) -> bool {
        if self.price <= 0 {
            return false;
        }
        // confidence / price <= max_confidence_bps / BPS_DIVISOR
        // => confidence * BPS_DIVISOR <= price * max_confidence_bps
        let price_u128 = self.price as u128;
        let conf_u128 = self.confidence as u128;
        match conf_u128.checked_mul(BPS_DIVISOR as u128) {
            Some(lhs) => match price_u128.checked_mul(max_confidence_bps as u128) {
                Some(rhs) => lhs <= rhs,
                None => false,
            },
            None => false,
        }
    }
}
