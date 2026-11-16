pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("4XadfZBdH5iRQ9fVHrffKaxyAseU2j7V5BB3v1oxNru2");

#[program]
pub mod ammq32026 {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        withdraw_lp_fee: u16,
        fee: u16,
        authority: Option<Pubkey>,
    ) -> Result<()> {
        ctx.accounts
            .init(seed, fee, withdraw_lp_fee, authority, ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        ctx.accounts.deposit(amount, max_x, max_y)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        ctx.accounts.withdraw(amount, min_x, min_y)
    }

    pub fn quote_lp(ctx: Context<Withdraw>, amount: u64) -> Result<Quote> {
        ctx.accounts.quote(amount)
    }

    pub fn swap(ctx: Context<Swap>, is_x: bool, amount_in: u64, min_out: u64) -> Result<()> {
        ctx.accounts.swap(is_x, amount_in, min_out)
    }

    pub fn quote_swap(ctx: Context<Swap>, amount: u64, is_x: bool) -> Result<Quote> {
        ctx.accounts.quote(amount, is_x)
    }
}
