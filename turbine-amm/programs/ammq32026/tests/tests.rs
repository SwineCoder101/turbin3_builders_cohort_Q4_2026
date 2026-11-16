use {
    anchor_lang::AccountDeserialize,
    anchor_spl::associated_token,
    litesvm::LiteSVM,
    litesvm_token::{get_spl_account, spl_token, CreateAssociatedTokenAccount, CreateMint, MintTo},
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

    /// A fresh trader (not the LP) with SOL and X/Y ATAs funded with `amount` each.
    fn new_trader(&mut self, amount: u64) -> Keypair {
        let trader = Keypair::new();
        self.svm.airdrop(&trader.pubkey(), 1_000_000_000).unwrap();
        for mint in [self.mint_x, self.mint_y] {
            let ata = CreateAssociatedTokenAccount::new(&mut self.svm, &self.payer, &mint)
                .owner(&trader.pubkey())
                .send()
                .unwrap();
            MintTo::new(&mut self.svm, &self.payer, &mint, &ata, amount)
                .send()
                .unwrap();
        }
        trader
    }

    fn swap(
        &mut self,
        trader: &Keypair,
        is_x: bool,
        amount_in: u64,
        min_out: u64,
    ) -> litesvm::types::TransactionResult {
        let ix = create_swap_ix(
            trader, self.mint_x, self.mint_y, self.config, self.vault_x, self.vault_y,
            is_x, amount_in, min_out,
        );
        send(&mut self.svm, &[ix], trader, &[trader])
    }
}

impl Pool {
    fn treasury(&self) -> Pubkey {
        treasury_pda(&self.config)
    }

    fn treasury_ata(&self, mint: &Pubkey) -> Pubkey {
        associated_token::get_associated_token_address(&self.treasury(), mint)
    }

    fn init_treasury(&mut self, authority: &Keypair, protocol_fee_bps: u16) -> litesvm::types::TransactionResult {
        let ix = create_initialize_treasury_ix(
            authority, self.mint_x, self.mint_y, self.config, protocol_fee_bps,
        );
        send(&mut self.svm, &[ix], authority, &[authority])
    }
}

fn ata_of(owner: &Keypair, mint: &Pubkey) -> Pubkey {
    associated_token::get_associated_token_address(&owner.pubkey(), mint)
}

/// Protocol's cut of each swap input (bps), out of the pool's 30 bps fee.
const PROTOCOL_FEE_BPS: u64 = 5;

/// Initialize the pool, its treasury, and make the first deposit.
fn setup_pool_with_deposit() -> Pool {
    let mut pool = setup_pool_without_treasury();
    let payer = pool.payer.insecure_clone();
    pool.init_treasury(&payer, PROTOCOL_FEE_BPS as u16)
        .expect("treasury init failed");
    pool
}

/// Initialize the pool and make the first deposit, so there is LP to withdraw.
/// No treasury yet: swaps on this pool must fail.
fn setup_pool_without_treasury() -> Pool {
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

// ---------- Swap ----------

// Matches `fee: 30` in ix_handlers/init.rs (0.30%).
const FEE_BPS: u64 = 30;
const TRADER_FUNDS: u64 = 100_000_000;

/// Independent re-implementation of the swap pricing, to check the program against:
/// take the fee off the input, then out = reserve_out - k / (reserve_in + input_after_fee).
fn expected_swap_out(reserve_in: u64, reserve_out: u64, amount_in: u64) -> u64 {
    let after_fee = amount_in as u128 * (10_000 - FEE_BPS as u128) / 10_000;
    let k = reserve_in as u128 * reserve_out as u128;
    (reserve_out as u128 - k / (reserve_in as u128 + after_fee)) as u64
}

#[test]
fn test_swap_x_for_y() {
    let mut pool = setup_pool_with_deposit();
    let trader = pool.new_trader(TRADER_FUNDS);

    let amount_in = 10_000_000;
    let expected_out = expected_swap_out(DEPOSIT_X, DEPOSIT_Y, amount_in);

    let res = pool.swap(&trader, true, amount_in, expected_out);
    assert!(res.is_ok(), "swap failed: {:?}", res.err());

    // Trader paid X and received Y.
    assert_eq!(token_balance(&pool.svm, &ata_of(&trader, &pool.mint_x)), TRADER_FUNDS - amount_in);
    assert_eq!(token_balance(&pool.svm, &ata_of(&trader, &pool.mint_y)), TRADER_FUNDS + expected_out);
    // The vault received the input minus the protocol's cut, and paid out Y.
    let cut = protocol_cut(amount_in);
    assert_eq!(token_balance(&pool.svm, &pool.vault_x), DEPOSIT_X + amount_in - cut);
    assert_eq!(token_balance(&pool.svm, &pool.vault_y), DEPOSIT_Y - expected_out);
    // The protocol's cut landed in the treasury, in the input token.
    assert_eq!(token_balance(&pool.svm, &pool.treasury_ata(&pool.mint_x)), cut);
    assert_eq!(token_balance(&pool.svm, &pool.treasury_ata(&pool.mint_y)), 0);
}

#[test]
fn test_swap_y_for_x() {
    let mut pool = setup_pool_with_deposit();
    let trader = pool.new_trader(TRADER_FUNDS);

    let amount_in = 10_000_000;
    let expected_out = expected_swap_out(DEPOSIT_Y, DEPOSIT_X, amount_in);

    let res = pool.swap(&trader, false, amount_in, expected_out);
    assert!(res.is_ok(), "swap failed: {:?}", res.err());

    assert_eq!(token_balance(&pool.svm, &ata_of(&trader, &pool.mint_y)), TRADER_FUNDS - amount_in);
    assert_eq!(token_balance(&pool.svm, &ata_of(&trader, &pool.mint_x)), TRADER_FUNDS + expected_out);
    let cut = protocol_cut(amount_in);
    assert_eq!(token_balance(&pool.svm, &pool.vault_y), DEPOSIT_Y + amount_in - cut);
    assert_eq!(token_balance(&pool.svm, &pool.vault_x), DEPOSIT_X - expected_out);
    assert_eq!(token_balance(&pool.svm, &pool.treasury_ata(&pool.mint_y)), cut);
    assert_eq!(token_balance(&pool.svm, &pool.treasury_ata(&pool.mint_x)), 0);
}

#[test]
fn test_swap_keeps_k_from_decreasing() {
    let mut pool = setup_pool_with_deposit();
    let trader = pool.new_trader(TRADER_FUNDS);
    let k_before = DEPOSIT_X as u128 * DEPOSIT_Y as u128;

    pool.swap(&trader, true, 10_000_000, 0).unwrap();

    let k_after = token_balance(&pool.svm, &pool.vault_x) as u128
        * token_balance(&pool.svm, &pool.vault_y) as u128;
    // The LPs' share of the fee stays in the pool, so k strictly grows.
    assert!(k_after > k_before, "k shrank: {k_before} -> {k_after}");
}

#[test]
fn test_swap_fails_when_slippage_exceeded() {
    let mut pool = setup_pool_with_deposit();
    let trader = pool.new_trader(TRADER_FUNDS);

    let amount_in = 10_000_000;
    let expected_out = expected_swap_out(DEPOSIT_X, DEPOSIT_Y, amount_in);

    // Demand 1 more than the pool will give.
    assert_anchor_error(pool.swap(&trader, true, amount_in, expected_out + 1), "SlippageExceeded");

    // Nothing moved.
    assert_eq!(token_balance(&pool.svm, &ata_of(&trader, &pool.mint_x)), TRADER_FUNDS);
    assert_eq!(token_balance(&pool.svm, &pool.vault_x), DEPOSIT_X);
    assert_eq!(token_balance(&pool.svm, &pool.vault_y), DEPOSIT_Y);
    assert_eq!(token_balance(&pool.svm, &pool.treasury_ata(&pool.mint_x)), 0);
}

#[test]
fn test_swap_fails_for_zero_amount() {
    let mut pool = setup_pool_with_deposit();
    let trader = pool.new_trader(TRADER_FUNDS);
    assert_anchor_error(pool.swap(&trader, true, 0, 0), "InvalidAmount");
}

#[test]
fn test_swap_fails_when_output_rounds_to_zero() {
    let mut pool = setup_pool_with_deposit();
    let trader = pool.new_trader(TRADER_FUNDS);
    // 1 base unit in: the fee leaves 0 to price, so the output would be 0.
    assert_anchor_error(pool.swap(&trader, true, 1, 0), "InvalidAmount");
}

#[test]
fn test_swap_fees_split_between_lps_and_treasury() {
    let mut pool = setup_pool_with_deposit();
    let trader = pool.new_trader(TRADER_FUNDS);

    // Trader swaps X -> Y, then swaps all that Y straight back to X.
    let x_in = 10_000_000;
    let y_received = expected_swap_out(DEPOSIT_X, DEPOSIT_Y, x_in);
    pool.swap(&trader, true, x_in, y_received).unwrap();
    pool.swap(&trader, false, y_received, 0).unwrap();

    let cut_x = protocol_cut(x_in); // protocol's cut of leg 1, paid in X
    let cut_y = protocol_cut(y_received); // protocol's cut of leg 2, paid in Y

    let trader_x = token_balance(&pool.svm, &ata_of(&trader, &pool.mint_x));
    let trader_loss = TRADER_FUNDS - trader_x;
    assert!(trader_loss > 0, "a round trip must cost the trader fees");

    // The treasury holds exactly the protocol's cut of each leg.
    let treasury_x = token_balance(&pool.svm, &pool.treasury_ata(&pool.mint_x));
    let treasury_y = token_balance(&pool.svm, &pool.treasury_ata(&pool.mint_y));
    assert_eq!(treasury_x, cut_x);
    assert_eq!(treasury_y, cut_y);

    // Conservation: every token is with the trader, in the vault, or in the treasury.
    let vault_x = token_balance(&pool.svm, &pool.vault_x);
    let vault_y = token_balance(&pool.svm, &pool.vault_y);
    let trader_y = token_balance(&pool.svm, &ata_of(&trader, &pool.mint_y));
    assert_eq!(trader_x + vault_x + treasury_x, TRADER_FUNDS + DEPOSIT_X);
    assert_eq!(trader_y + vault_y + treasury_y, TRADER_FUNDS + DEPOSIT_Y);

    // The LP (sole liquidity provider) withdraws everything.
    pool.withdraw(DEPOSIT_LP, 0, 0).unwrap();
    let lp_x = token_balance(&pool.svm, &pool.user_ata(&pool.mint_x));
    let lp_y = token_balance(&pool.svm, &pool.user_ata(&pool.mint_y));
    assert_eq!(lp_x, MINTED_TO_USER + trader_loss - cut_x);
    assert_eq!(lp_y, MINTED_TO_USER - cut_y);

    // At the ~1:1 pool price, the LPs' share of the fees (25 bps per leg)
    // is worth far more than the protocol's (5 bps per leg).
    let lp_gain = (lp_x + lp_y) - 2 * MINTED_TO_USER;
    assert!(lp_gain > treasury_x + treasury_y, "LPs should earn more than the protocol");
}

// ---------- Treasury ----------

fn protocol_cut(amount_in: u64) -> u64 {
    amount_in * PROTOCOL_FEE_BPS / 10_000
}

fn read_treasury(pool: &Pool) -> ammq32026::Treasury {
    let account = pool.svm.get_account(&pool.treasury()).expect("treasury not found");
    ammq32026::Treasury::try_deserialize(&mut account.data.as_slice()).unwrap()
}

#[test]
fn test_initialize_treasury() {
    let pool = setup_pool_with_deposit();

    let treasury = read_treasury(&pool);
    assert_eq!(treasury.config, pool.config);
    assert_eq!(treasury.authority, pool.payer.pubkey());
    assert_eq!(treasury.protocol_fee_bps as u64, PROTOCOL_FEE_BPS);

    // Both treasury token accounts exist, owned by the treasury PDA, and start empty.
    for mint in [pool.mint_x, pool.mint_y] {
        let ata = get_spl_account::<spl_token::state::Account>(&pool.svm, &pool.treasury_ata(&mint)).unwrap();
        assert_eq!(ata.owner, pool.treasury());
        assert_eq!(ata.amount, 0);
    }
}

#[test]
fn test_initialize_treasury_fails_for_non_authority() {
    let mut pool = setup_pool_without_treasury();
    let stranger = Keypair::new();
    pool.svm.airdrop(&stranger.pubkey(), 1_000_000_000).unwrap();

    assert_anchor_error(pool.init_treasury(&stranger, 5), "InvalidAuthority");
}

#[test]
fn test_initialize_treasury_fails_when_protocol_fee_exceeds_pool_fee() {
    let mut pool = setup_pool_without_treasury();
    let payer = pool.payer.insecure_clone();

    // Pool fee is 30 bps; the protocol can take at most all of it.
    assert_anchor_error(pool.init_treasury(&payer, 31), "ProtocolFeeTooHigh");
    assert!(pool.init_treasury(&payer, 30).is_ok());
}

#[test]
fn test_initialize_treasury_twice_fails() {
    let mut pool = setup_pool_with_deposit();
    let payer = pool.payer.insecure_clone();
    // `init` refuses an account that already exists, so the fee can't be reset this way.
    assert!(pool.init_treasury(&payer, 0).is_err());
    assert_eq!(read_treasury(&pool).protocol_fee_bps as u64, PROTOCOL_FEE_BPS);
}

#[test]
fn test_swap_fails_without_treasury() {
    let mut pool = setup_pool_without_treasury();
    let trader = pool.new_trader(TRADER_FUNDS);
    assert_anchor_error(pool.swap(&trader, true, 10_000_000, 0), "AccountNotInitialized");
}
