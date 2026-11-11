use anchor_lang::prelude::*;

use crate::error::EscrowError;

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub seed: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive: u64,
    pub bump: u8,
    pub expiration: Option<i64>,
}

impl Escrow {
    /// True once the on-chain clock has reached the expiration time.
    pub fn is_expired(&self, now: i64) -> bool {
        matches!(self.expiration, Some(expiration) if now >= expiration)
    }

    /// A new expiration must be in the future (or `None` to remove it).
    pub fn validate_expiration(expiration: Option<i64>, now: i64) -> Result<()> {
        if let Some(expiration) = expiration {
            require_gt!(expiration, now, EscrowError::InvalidExpiration);
        }
        Ok(())
    }
}
