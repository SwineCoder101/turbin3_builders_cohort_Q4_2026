use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use constant_product_curve::ConstantProduct;

use crate::{
    error::AmmError,
    state::{Config, Treasury},
    Quote,
};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub trader: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(mut,
        associated_token::mint = mint_x,
        associated_token::authority= trader,
    )]
    pub trader_ata_x: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        associated_token::mint = mint_y,
        associated_token::authority = trader
    )]
    pub trader_ata_y: Box<Account<'info, TokenAccount>>,

    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(mut,
        associated_token::mint = mint_x,
        associated_token::authority= config,
    )]
    pub vault_ata_x: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_ata_y: Box<Account<'info, TokenAccount>>,

    // Required: a pool can't be swapped until its treasury is initialized, so a
    // trader can never skip the protocol fee by leaving the treasury out.
    #[account(
        has_one = config,
        seeds = [b"treasury", config.key().as_ref()],
        bump = treasury.bump,
    )]
    pub treasury: Box<Account<'info, Treasury>>,

    #[account(mut,
        associated_token::mint = mint_x,
        associated_token::authority = treasury,
    )]
    pub treasury_ata_x: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        associated_token::mint = mint_y,
        associated_token::authority = treasury,
    )]
    pub treasury_ata_y: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Swap<'info> {
    pub fn swap(&self, is_x: bool, amount_in: u64, min_out: u64) -> Result<()> {
        require!(!self.config.locked, AmmError::PoolLocked);

        let quote = self.quote(amount_in, is_x)?;
        let amount_out = if is_x { quote.y } else { quote.x };
        require_gte!(amount_out, min_out, AmmError::SlippageExceeded);

        // Split the input: the protocol's cut goes to the treasury, the rest
        // (including the LPs' share of the fee) goes into the vault.
        let protocol_fee = self.protocol_fee(amount_in)?;
        let to_vault = amount_in
            .checked_sub(protocol_fee)
            .ok_or(AmmError::Underflow)?;

        // trader → vault
        self.deposit_tokens(is_x, to_vault)?;
        // trader → treasury
        if protocol_fee > 0 {
            self.pay_protocol_fee(is_x, protocol_fee)?;
        }
        // vault → trader: the other token
        self.withdraw_tokens(amount_out, !is_x)
    }

    /// The protocol's cut of a swap input, rounded down.
    /// `protocol_fee_bps <= config.fee` (enforced at treasury init), so this is
    /// always part of the fee the trader already pays, never an extra charge.
    pub fn protocol_fee(&self, amount_in: u64) -> Result<u64> {
        let fee = (amount_in as u128)
            .checked_mul(self.treasury.protocol_fee_bps as u128)
            .ok_or(AmmError::Overflow)?
            / 10_000;
        Ok(fee as u64)
    }

    /// trader → treasury, in the input token. The trader signs, so no PDA seeds needed.
    pub fn pay_protocol_fee(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.trader_ata_x.to_account_info(),
                self.treasury_ata_x.to_account_info(),
            ),
            false => (
                self.trader_ata_y.to_account_info(),
                self.treasury_ata_y.to_account_info(),
            ),
        };

        let ctx = CpiContext::new(
            self.token_program.key(),
            Transfer {
                from,
                to,
                authority: self.trader.to_account_info(),
            },
        );

        transfer(ctx, amount)
    }

    pub fn withdraw_tokens(&self, amount: u64, is_x: bool) -> Result<()> {
        let cpi_program = self.token_program.key();

        let cpi_accounts = match is_x {
            true => Transfer {
                from: self.vault_ata_x.to_account_info(),
                to: self.trader_ata_x.to_account_info(),
                authority: self.config.to_account_info(),
            },
            false => Transfer {
                from: self.vault_ata_y.to_account_info(),
                to: self.trader_ata_y.to_account_info(),
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

    pub fn deposit_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.trader_ata_x.to_account_info(),
                self.vault_ata_x.to_account_info(),
            ),
            false => (
                self.trader_ata_y.to_account_info(),
                self.vault_ata_y.to_account_info(),
            ),
        };

        let cpi_program = self.token_program.key();

        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.trader.to_account_info(),
        };

        let ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(ctx, amount)
    }

    /// Price a swap at the current reserves, after the pool fee.
    pub fn quote(&self, amount: u64, is_x: bool) -> Result<Quote> {
        require_neq!(amount, 0, AmmError::InvalidAmount);

        let x = self.vault_ata_x.amount;
        let y = self.vault_ata_y.amount;
        require!(x > 0 && y > 0, AmmError::NoLiquidityInPool);

        // The fee is taken from the input before pricing; it stays in the vault.
        let fee_multiplier = 10_000u16
            .checked_sub(self.config.fee)
            .ok_or(AmmError::InvalidFee)? as u128;
        let amount_after_fee = ((amount as u128) * fee_multiplier / 10_000) as u64;

        let amount_out = match is_x {
            true => ConstantProduct::delta_y_from_x_swap_amount(x, y, amount_after_fee), // X in → Y out
            false => ConstantProduct::delta_x_from_y_swap_amount(x, y, amount_after_fee), // Y in → X out
        }
        .map_err(AmmError::from)?;

        // A tiny input can round to 0 output: don't let the trader pay for nothing.
        require!(amount_out > 0, AmmError::InvalidAmount);

        Ok(match is_x {
            true => Quote {
                x: amount,
                y: amount_out,
            },
            false => Quote {
                x: amount_out,
                y: amount,
            },
        })
    }
}
