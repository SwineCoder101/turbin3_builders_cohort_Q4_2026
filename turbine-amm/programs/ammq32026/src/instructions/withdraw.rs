use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer},
};
use constant_product_curve::ConstantProduct;

use crate::{error::AmmError, state::Config, Quote, PRECISION};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp: Account<'info, Mint>,

    // Must already exist and hold LP: you can't withdraw without LP tokens.
    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = user
    )]
    pub user_lp: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_x,
        associated_token::authority = user
    )]
    pub user_ata_x: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_y,
        associated_token::authority = user
    )]
    pub user_ata_y: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, lp_amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        require!(!self.config.locked, AmmError::PoolLocked);
        require_neq!(lp_amount, 0, AmmError::InvalidAmount);

        let quote = self.quote(lp_amount)?;
        require!(
            quote.x >= min_x && quote.y >= min_y,
            AmmError::SlippageExceeded
        );

        self.withdraw_tokens(quote.x, true)?;
        self.withdraw_tokens(quote.y, false)?;
        self.burn_lp(lp_amount)
    }

    pub fn quote(&self, lp_amount: u64) -> Result<Quote> {
        require_neq!(lp_amount, 0, AmmError::InvalidAmount);
        // The curve computes `l - a` unchecked, so reject a > l before calling it.
        require_gte!(
            self.mint_lp.supply,
            lp_amount,
            AmmError::InsufficientBalance
        );

        let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount, // x reserve
            self.vault_y.amount, // y reserve
            self.mint_lp.supply, // total LP
            lp_amount,           // LP being burned
            PRECISION,
        )
        .map_err(AmmError::from)?;

        Ok(Quote {
            x: amounts.x,
            y: amounts.y,
        })
    }

    pub fn withdraw_tokens(&self, amount: u64, is_x: bool) -> Result<()> {
        let cpi_program = self.token_program.key();

        let cpi_accounts = match is_x {
            true => Transfer {
                from: self.vault_x.to_account_info(),
                to: self.user_ata_x.to_account_info(),
                authority: self.config.to_account_info(),
            },
            false => Transfer {
                from: self.vault_y.to_account_info(),
                to: self.user_ata_y.to_account_info(),
                authority: self.config.to_account_info(),
            },
        };

        // The vaults are owned by the config PDA, so the program signs with its seeds.
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"config",
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ]];

        let ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(ctx, amount)
    }

    pub fn burn_lp(&self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.key();
        let cpi_accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let ctx = CpiContext::new(cpi_program, cpi_accounts);

        burn(ctx, amount)
    }
}
