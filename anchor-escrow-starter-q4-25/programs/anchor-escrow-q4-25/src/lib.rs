use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;

pub use instructions::*;
pub use state::*;

declare_id!("5UFZzEt5vU9fxtUAgsD11z63ApZEHJ5bH7Z4QpFwZ2CQ");

#[program]
pub mod anchor_escrow_q4_25 {
    use super::*;
    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.deposit(deposit)?;
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }

    pub fn take(ctx: Context<Take>, expected_receive: u64) -> Result<()> {
        // Slippage guard: fail if the maker changed the price after the taker saw it.
        require_eq!(
            ctx.accounts.escrow.receive,
            expected_receive,
            error::EscrowError::PriceChanged
        );
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close_vault()
    }

    pub fn update(ctx: Context<Update>, new_receive: u64) -> Result<()> {
        ctx.accounts.update_receive(new_receive)
    }
}
