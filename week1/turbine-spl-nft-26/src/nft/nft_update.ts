import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import wallet from "../../devnet-wallet.json";
import {
  createSignerFromKeypair,
  publicKey,
  signerIdentity,
} from "@metaplex-foundation/umi";
import { fetchAsset, mplCore, update } from "@metaplex-foundation/mpl-core";
import { base58 } from "@metaplex-foundation/umi/serializers";

const umi = createUmi(
  process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
);

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(signerIdentity(signer));
umi.use(mplCore());

//paste the address of an asset your wallet is the update authority of
const assetAddress = publicKey("kARw6p4ZnVXz4SyjrzRF9azFCcT4Mks3Ym1YSLv7CBX");

//the new values; leave uri as-is or point it at a freshly uploaded metadata JSON
const newName = "swinecoder v2";
const newUri =
  "https://gateway.irys.xyz/EUU2NT4BTj6k26g3k3gVKkYQ1ZN5P1ZtsGKU5yTjV5AS";

(async () => {
  try {
    const asset = await fetchAsset(umi, assetAddress);

    // the update authority (not the owner) controls metadata, and it stays
    // with the creator even after the asset is transferred to someone else
    if (
      asset.updateAuthority.type !== "Address" ||
      asset.updateAuthority.address !== signer.publicKey
    ) {
      throw new Error("your wallet is not the update authority of this asset");
    }

    console.log(`owner:  ${asset.owner}`);
    console.log(`before: ${asset.name} -> ${asset.uri}`);

    const tx = await update(umi, {
      asset,
      name: newName,
      uri: newUri,
    }).sendAndConfirm(umi);

    const signature = base58.deserialize(tx.signature)[0];

    const updated = await fetchAsset(umi, assetAddress);
    console.log(`after:  ${updated.name} -> ${updated.uri}`);
    console.log(`update txid: ${signature}`);
  } catch (e) {
    console.log(`error ${e}`);
  }
})();
