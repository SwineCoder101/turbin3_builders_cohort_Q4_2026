use {
    anchor_lang::{
        solana_program::instruction::Instruction, system_program::ID as SYSTEM_PROGRAM_ID,
        InstructionData, ToAccountMetas,
    },
    anchor_spl::associated_token::{self, ID as ASSOCIATED_TOKEN_PROGRAM_ID},
    litesvm_token::spl_token::ID as TOKEN_PROGRAM_ID,
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
};

/// Builds a `swap` instruction: the trader sends `amount_in` of X (`is_x = true`)
/// or Y (`is_x = false`) and must receive at least `min_out` of the other token.
/// The trader's X and Y ATAs must already exist.
pub fn create_swap_ix(
    trader: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    config: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
    is_x: bool,
    amount_in: u64,
    min_out: u64,
) -> Instruction {
    let trader_pk = trader.pubkey();

    Instruction::new_with_bytes(
        ammq32026::id(),
        &ammq32026::instruction::Swap {
            is_x,
            amount_in,
            min_out,
        }
        .data(),
        ammq32026::accounts::Swap {
            trader: trader_pk,
            mint_x,
            mint_y,
            trader_ata_x: associated_token::get_associated_token_address(&trader_pk, &mint_x),
            trader_ata_y: associated_token::get_associated_token_address(&trader_pk, &mint_y),
            config,
            vault_ata_x: vault_x,
            vault_ata_y: vault_y,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}
