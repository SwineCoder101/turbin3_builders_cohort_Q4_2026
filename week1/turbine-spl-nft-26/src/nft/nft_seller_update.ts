// Demo: a seller can still change an NFT's metadata after selling it.
//
// 1. seller mints a Metaplex Core NFT
// 2. seller transfers it to a brand-new buyer wallet (the "sale")
// 3. seller updates the name + uri of the NFT the buyer now owns
// 4. buyer tries to update it and is rejected
//
// Ownership and update authority are separate roles in Core. Transferring
// moves ownership only; the update authority stays with the creator unless
// it is explicitly handed over or removed.

import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import wallet from "../../devnet-wallet.json";
import {
  createSignerFromKeypair,
  generateSigner,
  signerIdentity,
} from "@metaplex-foundation/umi";
import {
  create,
  fetchAsset,
  mplCore,
  transfer,
  update,
} from "@metaplex-foundation/mpl-core";
import { base58 } from "@metaplex-foundation/umi/serializers";

const umi = createUmi(
  process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
);

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const seller = createSignerFromKeypair(umi, keypair);

umi.use(signerIdentity(seller));
umi.use(mplCore());

const originalUri =
  "https://gateway.irys.xyz/EUU2NT4BTj6k26g3k3gVKkYQ1ZN5P1ZtsGKU5yTjV5AS";

// in a real RWA flow this would point at new metadata, e.g. an updated valuation
const changedUri = `${originalUri}?changed-after-sale=${Date.now()}`;

const sig = (s: Uint8Array) => base58.deserialize(s)[0];

const show = async (label: string, address: typeof asset.publicKey) => {
  const a = await fetchAsset(umi, address);
  console.log(`  ${label}`);
  console.log(`    owner:            ${a.owner}`);
  console.log(`    update authority: ${a.updateAuthority.address}`);
  console.log(`    name:             ${a.name}`);
  console.log(`    uri:              ${a.uri}`);
  return a;
};

const asset = generateSigner(umi);

(async () => {
  try {
    const buyer = generateSigner(umi);
    console.log(`seller: ${seller.publicKey}`);
    console.log(`buyer:  ${buyer.publicKey}  (fresh wallet)\n`);

    // 1. mint
    const mintTx = await create(umi, {
      asset,
      name: "Deed #1",
      uri: originalUri,
    }).sendAndConfirm(umi);
    console.log(`1. seller minted ${asset.publicKey}`);
    console.log(`   tx: ${sig(mintTx.signature)}`);
    await show("after mint", asset.publicKey);

    // 2. sell (transfer) to the buyer
    const transferTx = await transfer(umi, {
      asset: await fetchAsset(umi, asset.publicKey),
      newOwner: buyer.publicKey,
    }).sendAndConfirm(umi);
    console.log(`\n2. seller transferred it to the buyer`);
    console.log(`   tx: ${sig(transferTx.signature)}`);
    await show("after sale", asset.publicKey);

    // 3. seller changes the metadata of an NFT they no longer own
    const updateTx = await update(umi, {
      asset: await fetchAsset(umi, asset.publicKey),
      name: "Deed #1 (changed by seller)",
      uri: changedUri,
    }).sendAndConfirm(umi);
    console.log(`\n3. seller updated the metadata AFTER the sale`);
    console.log(`   tx: ${sig(updateTx.signature)}`);
    const after = await show("after seller update", asset.publicKey);

    const stillBuyers = after.owner === buyer.publicKey;
    const changed = after.uri === changedUri;
    console.log(
      `   buyer still owns it: ${stillBuyers}, metadata changed: ${changed}`,
    );

    // 4. the owner (buyer) cannot do the same. Seller pays the fee so the
    //    buyer wallet doesn't need SOL; the buyer signs as the authority.
    console.log(`\n4. buyer tries to update it`);
    try {
      await update(umi, {
        asset: await fetchAsset(umi, asset.publicKey),
        name: "Deed #1 (changed by buyer)",
        authority: buyer,
        payer: seller,
      }).sendAndConfirm(umi);
      console.log("   buyer update SUCCEEDED (unexpected)");
    } catch (e) {
      const msg = String(e).split("\n")[0];
      console.log(`   buyer update rejected as expected: ${msg}`);
    }

    console.log(
      `\nResult: owning a Core NFT does not mean controlling its metadata.`,
    );
    console.log(
      `To lock metadata at sale time, set the update authority to None before transferring.`,
    );
  } catch (e) {
    console.log(`error ${e}`);
  }
})();
