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

/// Builds a `withdraw` instruction: burn `lp_amount` LP and receive at least
/// `min_x` / `min_y` back. The user's X/Y/LP token accounts are the ATAs created
/// by the deposit handler.
pub fn create_withdraw_ix(
    payer: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    mint_lp: Pubkey,
    config: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
    lp_amount: u64,
    min_x: u64,
    min_y: u64,
) -> Instruction {
    let user = payer.pubkey();

    let user_lp = associated_token::get_associated_token_address(&user, &mint_lp);
    let user_ata_x = associated_token::get_associated_token_address(&user, &mint_x);
    let user_ata_y = associated_token::get_associated_token_address(&user, &mint_y);

    Instruction::new_with_bytes(
        ammq32026::id(),
        &ammq32026::instruction::Withdraw {
            amount: lp_amount,
            min_x,
            min_y,
        }
        .data(),
        ammq32026::accounts::Withdraw {
            user,
            mint_x,
            mint_y,
            config,
            mint_lp,
            user_lp,
            user_ata_x,
            user_ata_y,
            vault_x,
            vault_y,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}
