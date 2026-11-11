use anchor_lang::prelude::*;

use crate::{error::EscrowError, Escrow};

#[derive(Accounts)]
pub struct Update<'info> {
    // Only the maker can change their own offer, so they must sign.
    pub maker: Signer<'info>,
    #[account(
        mut,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), &escrow.seed.to_le_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,
}

impl<'info> Update<'info> {
    pub fn update_receive(&mut self, new_receive: u64) -> Result<()> {
        require_gt!(new_receive, 0, EscrowError::InvalidAmount);
        self.escrow.receive = new_receive;
        Ok(())
    }
}
