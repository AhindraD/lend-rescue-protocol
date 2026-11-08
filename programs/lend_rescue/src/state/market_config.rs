use anchor_lang::prelude::*;

#[account]
pub struct MarketConfig {
    /// Authority who initialized the market and can pause/unpause
    pub authority: Pubkey,
    /// Unique market identifier
    pub market_id: u64,
    /// Permitted collateral token mint
    pub collateral_mint: Pubkey,
    /// Permitted debt token mint
    pub debt_mint: Pubkey,
    /// PDA vault holding deposited collateral tokens
    pub collateral_vault: Pubkey,
    /// Collateral token decimals
    pub collateral_decimals: u8,
    /// Debt token decimals
    pub debt_decimals: u8,
    /// PDA reference to the collateral mock oracle
    pub collateral_oracle: Pubkey,
    /// PDA reference to the debt mock oracle
    pub debt_oracle: Pubkey,
    /// Liquidation threshold in basis points (e.g. 8500 = 85%)
    pub liquidation_threshold_bps: u16,
    /// Default demo swap pool fee in basis points (e.g. 30 = 0.3%)
    pub demo_swap_fee_bps: u16,
    /// Emergency pause switch
    pub is_paused: bool,
    /// Schema version
    pub version: u8,
    /// PDA bump
    pub bump: u8,
}

impl MarketConfig {
    pub const LEN: usize = 8 // discriminator
        + 32 // authority
        + 8  // market_id
        + 32 // collateral_mint
        + 32 // debt_mint
        + 32 // collateral_vault
        + 1  // collateral_decimals
        + 1  // debt_decimals
        + 32 // collateral_oracle
        + 32 // debt_oracle
        + 2  // liquidation_threshold_bps
        + 2  // demo_swap_fee_bps
        + 1  // is_paused
        + 1  // version
        + 1  // bump
        + 64; // reserved padding
}
