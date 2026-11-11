import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorEscrowQ425 } from "../target/types/anchor_escrow_q4_25";
import { expect } from "chai";
import { getAssociatedTokenAddressSync, createAssociatedTokenAccountInstruction, createMint, mintTo, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";

describe("anchor_escrow_q4_25", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorEscrowQ425 as Program<AnchorEscrowQ425>;

  const maker = provider.wallet.publicKey;
  const taker = anchor.web3.Keypair.generate();

  let mintA: anchor.web3.PublicKey;
  let mintB: anchor.web3.PublicKey;
  let makerAtaA: anchor.web3.PublicKey;
  let takerAtaB: anchor.web3.PublicKey;
  let makerAtaB: anchor.web3.PublicKey;
  let takerAtaA: anchor.web3.PublicKey;

  const seed = new anchor.BN(1234);
  let escrowPda: anchor.web3.PublicKey;
  let escrowBump: number;
  let vault: anchor.web3.PublicKey;

  const depositAmount = 100;
  const receiveAmount = 200;

  before(async () => {
    // Airdrop SOL to maker and taker
    await provider.connection.requestAirdrop(maker, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(taker.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Create mints (decimals=0 for simplicity)
    mintA = await createMint(provider.connection, provider.wallet.payer, maker, null, 0);
    mintB = await createMint(provider.connection, provider.wallet.payer, taker.publicKey, null, 0);

    // Create ATAs and mint tokens
    makerAtaA = getAssociatedTokenAddressSync(mintA, maker);
    const makerAtaATx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(provider.wallet.publicKey, makerAtaA, maker, mintA)
    );
    await provider.sendAndConfirm(makerAtaATx);
    await mintTo(provider.connection, provider.wallet.payer, mintA, makerAtaA, provider.wallet.payer, depositAmount * 10);

    takerAtaB = getAssociatedTokenAddressSync(mintB, taker.publicKey);
    const takerAtaBTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(taker.publicKey, takerAtaB, taker.publicKey, mintB)
    );
    await provider.sendAndConfirm(takerAtaBTx, [taker]);
    await mintTo(provider.connection, taker, mintB, takerAtaB, taker, receiveAmount * 10);

  });

  it("Makes and refunds the escrow", async () => {
    const seed1 = new anchor.BN(1111);
    [escrowPda, escrowBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.toBuffer(), seed1.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    vault = getAssociatedTokenAddressSync(mintA, escrowPda, true);

    // Make
    await program.methods
      .make(seed1, new anchor.BN(depositAmount), new anchor.BN(receiveAmount), null)
      .accountsStrict({
        maker: maker,
        mintA: mintA,
        mintB: mintB,
        makerAtaA: makerAtaA,
        escrow: escrowPda,
        vault: vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const escrowAccount = await program.account.escrow.fetch(escrowPda);
    expect(escrowAccount.maker.toBase58()).to.equal(maker.toBase58());
    expect(escrowAccount.mintA.toBase58()).to.equal(mintA.toBase58());
    expect(escrowAccount.mintB.toBase58()).to.equal(mintB.toBase58());
    expect(escrowAccount.receive.toNumber()).to.equal(receiveAmount);
    expect(escrowAccount.bump).to.equal(escrowBump);

    const vaultBalance = (await provider.connection.getTokenAccountBalance(vault)).value.uiAmount;
    expect(vaultBalance).to.equal(depositAmount);

    // Refund
    await program.methods
      .refund()
      .accountsStrict({
        maker: maker,
        mintA: mintA,
        makerAtaA: makerAtaA,
        escrow: escrowPda,
        vault: vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Check closed
    const escrowInfo = await provider.connection.getAccountInfo(escrowPda);
    expect(escrowInfo).to.be.null;

    const vaultInfo = await provider.connection.getAccountInfo(vault);
    expect(vaultInfo).to.be.null;
  });

  it("Makes and takes the escrow", async () => {
    const seed2 = new anchor.BN(2222);
    [escrowPda, escrowBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.toBuffer(), seed2.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    vault = getAssociatedTokenAddressSync(mintA, escrowPda, true);

    // Make (again for take path)
    await program.methods
      .make(seed2, new anchor.BN(depositAmount), new anchor.BN(receiveAmount), null)
      .accountsStrict({
        maker: maker,
        mintA: mintA,
        mintB: mintB,
        makerAtaA: makerAtaA,
        escrow: escrowPda,
        vault: vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Setup for take
    takerAtaA = getAssociatedTokenAddressSync(mintA, taker.publicKey);
    makerAtaB = getAssociatedTokenAddressSync(mintB, maker);

    // Take
    await program.methods
      .take(new anchor.BN(receiveAmount))
      .accountsStrict({
        taker: taker.publicKey,
        maker: maker,
        mintA: mintA,
        mintB: mintB,
        takerAtaA: takerAtaA,
        takerAtaB: takerAtaB,
        makerAtaB: makerAtaB,
        escrow: escrowPda,
        vault: vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([taker])
      .rpc();

    // Check closed
    const escrowInfo = await provider.connection.getAccountInfo(escrowPda);
    expect(escrowInfo).to.be.null;

    const vaultInfo = await provider.connection.getAccountInfo(vault);
    expect(vaultInfo).to.be.null;

    // Check balances
    const takerBalanceA = (await provider.connection.getTokenAccountBalance(takerAtaA)).value.uiAmount;
    expect(takerBalanceA).to.equal(depositAmount);

    const makerBalanceB = (await provider.connection.getTokenAccountBalance(makerAtaB)).value.uiAmount;
    expect(makerBalanceB).to.equal(receiveAmount);
  });

  it("Updates the escrow price, guards take against stale prices, then takes", async () => {
    const seed3 = new anchor.BN(3333);
    [escrowPda, escrowBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.toBuffer(), seed3.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    vault = getAssociatedTokenAddressSync(mintA, escrowPda, true);
    const newReceiveAmount = 150;

    // Balances before (takerAtaA / makerAtaB already exist from the previous test)
    const takerABefore = (await provider.connection.getTokenAccountBalance(takerAtaA)).value.uiAmount;
    const makerBBefore = (await provider.connection.getTokenAccountBalance(makerAtaB)).value.uiAmount;

    // Make
    await program.methods
      .make(seed3, new anchor.BN(depositAmount), new anchor.BN(receiveAmount), null)
      .accountsStrict({
        maker: maker,
        mintA: mintA,
        mintB: mintB,
        makerAtaA: makerAtaA,
        escrow: escrowPda,
        vault: vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Someone other than the maker cannot update the price
    try {
      await program.methods
        .update(new anchor.BN(1))
        .accountsStrict({ maker: taker.publicKey, escrow: escrowPda })
        .signers([taker])
        .rpc();
      expect.fail("non-maker update should have failed");
    } catch (err) {
      expect(err).to.be.instanceOf(anchor.AnchorError);
    }

    // A zero price is rejected
    try {
      await program.methods
        .update(new anchor.BN(0))
        .accountsStrict({ maker: maker, escrow: escrowPda })
        .rpc();
      expect.fail("zero price update should have failed");
    } catch (err) {
      expect((err as anchor.AnchorError).error.errorCode.code).to.equal("InvalidAmount");
    }

    // Maker updates the price
    await program.methods
      .update(new anchor.BN(newReceiveAmount))
      .accountsStrict({ maker: maker, escrow: escrowPda })
      .rpc();

    const escrowAccount = await program.account.escrow.fetch(escrowPda);
    expect(escrowAccount.receive.toNumber()).to.equal(newReceiveAmount);

    const takeAccounts = {
      taker: taker.publicKey,
      maker: maker,
      mintA: mintA,
      mintB: mintB,
      takerAtaA: takerAtaA,
      takerAtaB: takerAtaB,
      makerAtaB: makerAtaB,
      escrow: escrowPda,
      vault: vault,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    };

    // Taker still expects the old price -> rejected
    try {
      await program.methods
        .take(new anchor.BN(receiveAmount))
        .accountsStrict(takeAccounts)
        .signers([taker])
        .rpc();
      expect.fail("take with stale price should have failed");
    } catch (err) {
      expect((err as anchor.AnchorError).error.errorCode.code).to.equal("PriceChanged");
    }

    // Taker takes at the new price
    await program.methods
      .take(new anchor.BN(newReceiveAmount))
      .accountsStrict(takeAccounts)
      .signers([taker])
      .rpc();

    expect(await provider.connection.getAccountInfo(escrowPda)).to.be.null;
    expect(await provider.connection.getAccountInfo(vault)).to.be.null;

    const takerAAfter = (await provider.connection.getTokenAccountBalance(takerAtaA)).value.uiAmount;
    const makerBAfter = (await provider.connection.getTokenAccountBalance(makerAtaB)).value.uiAmount;
    expect(takerAAfter - takerABefore).to.equal(depositAmount);
    expect(makerBAfter - makerBBefore).to.equal(newReceiveAmount);
  });

  // ---------- Expiration ----------

  // Reads unix_timestamp straight from the Clock sysvar: the same value the program sees.
  // Clock layout: slot u64 | epoch_start_timestamp i64 | epoch u64 | leader_schedule_epoch u64 | unix_timestamp i64
  const chainNow = async (): Promise<number> => {
    const info = await provider.connection.getAccountInfo(anchor.web3.SYSVAR_CLOCK_PUBKEY);
    return Number(info!.data.readBigInt64LE(32));
  };

  const waitUntilChainTime = async (target: number) => {
    while ((await chainNow()) < target) {
      await new Promise((resolve) => setTimeout(resolve, 400));
    }
  };

  const escrowAccountsFor = (seed: anchor.BN) => {
    const [pda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    return { escrow: pda, vault: getAssociatedTokenAddressSync(mintA, pda, true) };
  };

  const makeEscrow = (seed: anchor.BN, expiration: number | null) => {
    const { escrow, vault } = escrowAccountsFor(seed);
    return program.methods
      .make(
        seed,
        new anchor.BN(depositAmount),
        new anchor.BN(receiveAmount),
        expiration === null ? null : new anchor.BN(expiration)
      )
      .accountsStrict({
        maker,
        mintA,
        mintB,
        makerAtaA,
        escrow,
        vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  };

  const takeEscrow = (seed: anchor.BN) => {
    const { escrow, vault } = escrowAccountsFor(seed);
    return program.methods
      .take(new anchor.BN(receiveAmount))
      .accountsStrict({
        taker: taker.publicKey,
        maker,
        mintA,
        mintB,
        takerAtaA,
        takerAtaB,
        makerAtaB,
        escrow,
        vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([taker])
      .rpc();
  };

  const expectError = async (promise: Promise<unknown>, code: string) => {
    try {
      await promise;
      expect.fail(`expected ${code}`);
    } catch (err) {
      expect((err as anchor.AnchorError).error?.errorCode.code, String(err)).to.equal(code);
    }
  };

  it("Rejects making an escrow that is already expired", async () => {
    const seed = new anchor.BN(4444);
    await expectError(makeEscrow(seed, (await chainNow()) - 10), "InvalidExpiration");
  });

  it("Blocks take after expiration; maker extends it and the taker can take", async () => {
    const seed = new anchor.BN(5555);
    const { escrow } = escrowAccountsFor(seed);
    const expiration = (await chainNow()) + 2;

    await makeEscrow(seed, expiration);
    expect((await program.account.escrow.fetch(escrow)).expiration.toNumber()).to.equal(expiration);

    await waitUntilChainTime(expiration);

    // Expired: taker cannot take
    await expectError(takeEscrow(seed), "EscrowExpired");

    // Maker cannot set an expiration in the past
    await expectError(
      program.methods
        .updateExpiration(new anchor.BN((await chainNow()) - 1))
        .accountsStrict({ maker, escrow })
        .rpc(),
      "InvalidExpiration"
    );

    // Maker extends the expiration by an hour
    const newExpiration = (await chainNow()) + 3600;
    await program.methods
      .updateExpiration(new anchor.BN(newExpiration))
      .accountsStrict({ maker, escrow })
      .rpc();
    expect((await program.account.escrow.fetch(escrow)).expiration.toNumber()).to.equal(newExpiration);

    // Taker can now take
    await takeEscrow(seed);
    expect(await provider.connection.getAccountInfo(escrow)).to.be.null;
  });

  it("Maker can refund an expired escrow", async () => {
    const seed = new anchor.BN(6666);
    const { escrow, vault } = escrowAccountsFor(seed);
    const expiration = (await chainNow()) + 2;

    await makeEscrow(seed, expiration);
    await waitUntilChainTime(expiration);
    await expectError(takeEscrow(seed), "EscrowExpired");

    const makerABefore = (await provider.connection.getTokenAccountBalance(makerAtaA)).value.uiAmount;

    await program.methods
      .refund()
      .accountsStrict({
        maker,
        mintA,
        makerAtaA,
        escrow,
        vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    expect(await provider.connection.getAccountInfo(escrow)).to.be.null;
    const makerAAfter = (await provider.connection.getTokenAccountBalance(makerAtaA)).value.uiAmount;
    expect(makerAAfter - makerABefore).to.equal(depositAmount);
  });

  it("Maker can remove the expiration entirely", async () => {
    const seed = new anchor.BN(7777);
    const { escrow } = escrowAccountsFor(seed);

    await makeEscrow(seed, (await chainNow()) + 3600);
    await program.methods.updateExpiration(null).accountsStrict({ maker, escrow }).rpc();
    expect((await program.account.escrow.fetch(escrow)).expiration).to.be.null;

    await takeEscrow(seed);
    expect(await provider.connection.getAccountInfo(escrow)).to.be.null;
  });
});
