use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("The escrow's receive amount changed since the taker last saw it")]
    PriceChanged,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
}
