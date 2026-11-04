import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import wallet from "../../devnet-wallet.json";
import {
  createSignerFromKeypair,
  publicKey,
  signerIdentity,
} from "@metaplex-foundation/umi";
import { burn, fetchAsset, mplCore } from "@metaplex-foundation/mpl-core";
import { base58 } from "@metaplex-foundation/umi/serializers";

const umi = createUmi(
  process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
);

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(signerIdentity(signer));
umi.use(mplCore());

//paste the address of an asset your wallet owns
const assetAddress = publicKey("JA76n67yJDT8gKbnvJdxJAijh1sRSehn8pL2gvXvULDZ");

const toSol = (lamports: bigint) => Number(lamports) / 1e9;

(async () => {
  try {
    const asset = await fetchAsset(umi, assetAddress);

    // only the owner (or a burn delegate) can destroy the asset
    if (asset.owner !== signer.publicKey) {
      throw new Error("your wallet does not own this asset, so it can't burn it");
    }

    const before = await umi.rpc.getAccount(assetAddress);
    const rentBefore = before.exists ? before.lamports.basisPoints : 0n;

    // burn wipes the asset's data and refunds its rent to the payer (us)
    const tx = await burn(umi, { asset }).sendAndConfirm(umi);

    const signature = base58.deserialize(tx.signature)[0];

    // Core shrinks the account to a 1-byte tombstone so the address can never
    // be reused. What stays behind is rent for that byte plus Metaplex's
    // protocol fee held in the account, which the owner can't withdraw.
    const after = await umi.rpc.getAccount(assetAddress);
    const rentAfter = after.exists ? after.lamports.basisPoints : 0n;
    const tombstoneRent = (await umi.rpc.getRent(1)).basisPoints;

    console.log(`asset rent before: ${toSol(rentBefore)} SOL`);
    console.log(`asset rent after:  ${toSol(rentAfter)} SOL`);
    console.log(`rent reclaimed:    ${toSol(rentBefore - rentAfter)} SOL`);
    console.log(`  left as 1-byte tombstone rent: ${toSol(tombstoneRent)} SOL`);
    console.log(`  left as Metaplex protocol fee: ${toSol(rentAfter - tombstoneRent)} SOL`);
    console.log(`burn txid: ${signature}`);
  } catch (e) {
    console.log(`error ${e}`);
  }
})();
