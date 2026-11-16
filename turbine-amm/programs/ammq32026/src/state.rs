use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub seed: u64,                 // Seed to be able to create different pools / configs
    pub authority: Option<Pubkey>, // If we want an authority to lock the config account
    pub mint_x: Pubkey,            // Token X
    pub mint_y: Pubkey,            // Token Y
    pub fee: u16,                  // Swap fee in basis points
    pub withdraw_lp_fee: u16,
    pub locked: bool,    // If the pool is locked
    pub config_bump: u8, // Bump seed for the config account
    pub lp_bump: u8,     // Bump seed for the LP token
}

/// Protocol treasury for one pool. PDA: ["treasury", config].
/// Owns the treasury token accounts (ATAs for mint_x and mint_y) that collect
/// the protocol's share of every swap fee.
#[account]
#[derive(InitSpace)]
pub struct Treasury {
    pub config: Pubkey,        // The pool this treasury belongs to
    pub authority: Pubkey,     // Who controls the treasury (the pool authority at init)
    pub protocol_fee_bps: u16, // Protocol's cut of each swap input, in bps; <= config.fee
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Quote {
    pub x: u64,
    pub y: u64,
}
