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

/// The pool's treasury PDA: ["treasury", config].
pub fn treasury_pda(config: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"treasury", config.as_ref()], &ammq32026::id()).0
}

/// Builds an `initialize_treasury` instruction signed by `authority`.
pub fn create_initialize_treasury_ix(
    authority: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    config: Pubkey,
    protocol_fee_bps: u16,
) -> Instruction {
    let treasury = treasury_pda(&config);

    Instruction::new_with_bytes(
        ammq32026::id(),
        &ammq32026::instruction::InitializeTreasury { protocol_fee_bps }.data(),
        ammq32026::accounts::InitializeTreasury {
            authority: authority.pubkey(),
            mint_x,
            mint_y,
            config,
            treasury,
            treasury_ata_x: associated_token::get_associated_token_address(&treasury, &mint_x),
            treasury_ata_y: associated_token::get_associated_token_address(&treasury, &mint_y),
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}
