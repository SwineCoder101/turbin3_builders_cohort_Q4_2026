import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import wallet from "../../devnet-wallet.json";
import {
  createSignerFromKeypair,
  publicKey,
  signerIdentity,
} from "@metaplex-foundation/umi";
import { fetchAsset, mplCore, transfer } from "@metaplex-foundation/mpl-core";
import { base58 } from "@metaplex-foundation/umi/serializers";

const umi = createUmi(
  process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
);

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(signerIdentity(signer));
umi.use(mplCore());

const assetAddress = publicKey("kARw6p4ZnVXz4SyjrzRF9azFCcT4Mks3Ym1YSLv7CBX");

const newOwner = publicKey("9EUd4VNcjMAysd7zQk3Q1a4tb28BYndLNBAQDiYnHJ64");

(async () => {
  try {
    // fetch the asset so the helper knows its current owner and plugins
    const asset = await fetchAsset(umi, assetAddress);
    console.log(`current owner: ${asset.owner}`);

    if (asset.owner !== signer.publicKey) {
      throw new Error("your wallet does not own this asset, so it can't transfer it");
    }

    // Core NFTs are a single account, so transferring just rewrites its owner field
    const tx = await transfer(umi, {
      asset,
      newOwner,
    }).sendAndConfirm(umi);

    const signature = base58.deserialize(tx.signature)[0];

    const updated = await fetchAsset(umi, assetAddress);
    console.log(`new owner: ${updated.owner}`);
    console.log(`transfer txid: ${signature}`);
  } catch (e) {
    console.log(`error ${e}`);
  }
})();
