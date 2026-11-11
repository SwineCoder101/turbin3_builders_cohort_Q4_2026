use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("The escrow's receive amount changed since the taker last saw it")]
    PriceChanged,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("The escrow has expired; the maker must refund it or update the expiration")]
    EscrowExpired,
    #[msg("Expiration must be in the future")]
    InvalidExpiration,
}
