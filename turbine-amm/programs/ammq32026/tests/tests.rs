use {
    anchor_spl::associated_token,
    litesvm::LiteSVM,
    litesvm_token::{get_spl_account, spl_token, CreateMint},
    solana_keypair::Keypair,
    solana_message::{Instruction, Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

mod ix_handlers;
use ix_handlers::*;

fn send(
    svm: &mut LiteSVM,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

// Setup function to initialize LiteSVM and create a payer keypair
fn setup() -> (
    LiteSVM,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
) {
    let program_id = ammq32026::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/ammq32026.so"
    ));

    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    // Create two mints (Mint A and Mint B) with 6 decimal places and the maker as the authority
    // This done using litesvm-token's CreateMint utility which creates the mint in the LiteSVM environment
    let mint_x = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let mint_y = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let config =
        Pubkey::find_program_address(&[b"config", &123u64.to_le_bytes()], &ammq32026::id()).0;
    let mint_lp = Pubkey::find_program_address(&[b"lp", config.as_ref()], &ammq32026::id()).0;

    // Derive the PDA for the vault associated token account using the config PDA and Mint A
    let vault_x = associated_token::get_associated_token_address(&config, &mint_x);
    let vault_y = associated_token::get_associated_token_address(&config, &mint_y);

    (
        svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    )
}

#[test]
fn test_initialize() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    let instruction = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );
    let res = send(&mut svm, &[instruction], &payer, &[&payer]);
    assert!(res.is_ok());
}

#[test]
pub fn test_deposit() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let res = send(&mut svm, &[init_ix, deposit_ix], &payer, &[&payer]);
    assert!(res.is_ok());
}

// ---------- Withdraw ----------

// Matches the amounts in ix_handlers/deposit.rs (first deposit into an empty pool).
const DEPOSIT_X: u64 = 200_000_000;
const DEPOSIT_Y: u64 = 200_000_000;
const DEPOSIT_LP: u64 = 100_000_000;
const MINTED_TO_USER: u64 = 1_000_000_000;

fn token_balance(svm: &LiteSVM, token_account: &Pubkey) -> u64 {
    get_spl_account::<spl_token::state::Account>(svm, token_account)
        .unwrap()
        .amount
}

fn mint_supply(svm: &LiteSVM, mint: &Pubkey) -> u64 {
    get_spl_account::<spl_token::state::Mint>(svm, mint)
        .unwrap()
        .supply
}

/// Assert a transaction failed with a specific Anchor error (e.g. "SlippageExceeded").
fn assert_anchor_error(res: litesvm::types::TransactionResult, code: &str) {
    let err = res.expect_err(&format!("expected {code}, but the transaction succeeded"));
    let needle = format!("Error Code: {code}");
    assert!(
        err.meta.logs.iter().any(|l| l.contains(&needle)),
        "expected {code}, got logs: {:#?}",
        err.meta.logs
    );
}

struct Pool {
    svm: LiteSVM,
    payer: Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    config: Pubkey,
    mint_lp: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
}

impl Pool {
    fn user_ata(&self, mint: &Pubkey) -> Pubkey {
        associated_token::get_associated_token_address(&self.payer.pubkey(), mint)
    }

    fn withdraw(&mut self, lp_amount: u64, min_x: u64, min_y: u64) -> litesvm::types::TransactionResult {
        let ix = create_withdraw_ix(
            &self.payer, self.mint_x, self.mint_y, self.mint_lp, self.config,
            self.vault_x, self.vault_y, lp_amount, min_x, min_y,
        );
        send(&mut self.svm, &[ix], &self.payer, &[&self.payer])
    }
}

/// Initialize the pool and make the first deposit, so there is LP to withdraw.
fn setup_pool_with_deposit() -> Pool {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    send(&mut svm, &[init_ix, deposit_ix], &payer, &[&payer]).expect("init + deposit failed");

    let pool = Pool { svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y };

    // Sanity-check the starting state.
    assert_eq!(token_balance(&pool.svm, &pool.vault_x), DEPOSIT_X);
    assert_eq!(token_balance(&pool.svm, &pool.vault_y), DEPOSIT_Y);
    assert_eq!(token_balance(&pool.svm, &pool.user_ata(&pool.mint_lp)), DEPOSIT_LP);
    assert_eq!(mint_supply(&pool.svm, &pool.mint_lp), DEPOSIT_LP);
    pool
}

#[test]
fn test_withdraw_half() {
    let mut pool = setup_pool_with_deposit();

    // Burn half the LP supply -> receive half of each vault.
    let lp_amount = DEPOSIT_LP / 2;
    let expected_x = DEPOSIT_X / 2;
    let expected_y = DEPOSIT_Y / 2;

    let res = pool.withdraw(lp_amount, expected_x, expected_y);
    assert!(res.is_ok(), "withdraw failed: {:?}", res.err());

    // User received X and Y, and their LP was burned.
    let user_x = pool.user_ata(&pool.mint_x);
    let user_y = pool.user_ata(&pool.mint_y);
    let user_lp = pool.user_ata(&pool.mint_lp);
    assert_eq!(token_balance(&pool.svm, &user_x), MINTED_TO_USER - DEPOSIT_X + expected_x);
    assert_eq!(token_balance(&pool.svm, &user_y), MINTED_TO_USER - DEPOSIT_Y + expected_y);
    assert_eq!(token_balance(&pool.svm, &user_lp), DEPOSIT_LP - lp_amount);

    // Vaults paid out, and the LP supply shrank by exactly the burned amount.
    assert_eq!(token_balance(&pool.svm, &pool.vault_x), DEPOSIT_X - expected_x);
    assert_eq!(token_balance(&pool.svm, &pool.vault_y), DEPOSIT_Y - expected_y);
    assert_eq!(mint_supply(&pool.svm, &pool.mint_lp), DEPOSIT_LP - lp_amount);
}

#[test]
fn test_withdraw_all() {
    let mut pool = setup_pool_with_deposit();

    let res = pool.withdraw(DEPOSIT_LP, DEPOSIT_X, DEPOSIT_Y);
    assert!(res.is_ok(), "withdraw failed: {:?}", res.err());

    // The only LP got everything back and the pool is empty.
    assert_eq!(token_balance(&pool.svm, &pool.user_ata(&pool.mint_x)), MINTED_TO_USER);
    assert_eq!(token_balance(&pool.svm, &pool.user_ata(&pool.mint_y)), MINTED_TO_USER);
    assert_eq!(token_balance(&pool.svm, &pool.vault_x), 0);
    assert_eq!(token_balance(&pool.svm, &pool.vault_y), 0);
    assert_eq!(mint_supply(&pool.svm, &pool.mint_lp), 0);
}

#[test]
fn test_withdraw_fails_when_slippage_exceeded() {
    let mut pool = setup_pool_with_deposit();

    // Half the LP pays out 100M X; demanding 1 more must fail.
    let res = pool.withdraw(DEPOSIT_LP / 2, DEPOSIT_X / 2 + 1, 0);
    assert_anchor_error(res, "SlippageExceeded");

    // Nothing moved.
    assert_eq!(token_balance(&pool.svm, &pool.vault_x), DEPOSIT_X);
    assert_eq!(mint_supply(&pool.svm, &pool.mint_lp), DEPOSIT_LP);
}

#[test]
fn test_withdraw_fails_for_zero_amount() {
    let mut pool = setup_pool_with_deposit();
    assert_anchor_error(pool.withdraw(0, 0, 0), "InvalidAmount");
}

#[test]
fn test_withdraw_fails_for_more_than_supply() {
    let mut pool = setup_pool_with_deposit();
    let res = pool.withdraw(DEPOSIT_LP + 1, 0, 0);
    assert_anchor_error(res, "InsufficientBalance");
    assert_eq!(token_balance(&pool.svm, &pool.vault_x), DEPOSIT_X);
}
