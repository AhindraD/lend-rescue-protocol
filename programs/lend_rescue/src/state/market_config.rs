use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
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
