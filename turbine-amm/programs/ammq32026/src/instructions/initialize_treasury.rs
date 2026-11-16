use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    error::AmmError,
    state::{Config, Treasury},
};

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub mint_x: Box<Account<'info, Mint>>,
    pub mint_y: Box<Account<'info, Mint>>,

    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(
        init,
        payer = authority,
        seeds = [b"treasury", config.key().as_ref()],
        bump,
        space = Treasury::DISCRIMINATOR.len() + Treasury::INIT_SPACE,
    )]
    pub treasury: Box<Account<'info, Treasury>>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = mint_x,
        associated_token::authority = treasury,
    )]
    pub treasury_ata_x: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = mint_y,
        associated_token::authority = treasury,
    )]
    pub treasury_ata_y: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeTreasury<'info> {
    pub fn init_treasury(
        &mut self,
        protocol_fee_bps: u16,
        bumps: &InitializeTreasuryBumps,
    ) -> Result<()> {
        let authority = self.config.authority.ok_or(AmmError::NoAuthoritySet)?;
        require_keys_eq!(authority, self.authority.key(), AmmError::InvalidAuthority);

        require!(
            protocol_fee_bps <= self.config.fee,
            AmmError::ProtocolFeeTooHigh
        );

        self.treasury.set_inner(Treasury {
            config: self.config.key(),
            authority,
            protocol_fee_bps,
            bump: bumps.treasury,
        });

        Ok(())
    }
}
