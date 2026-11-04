import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  createGenericFile,
  createSignerFromKeypair,
  signerIdentity,
} from "@metaplex-foundation/umi";
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";
import { readFile } from "fs/promises";

import wallet from "../../devnet-wallet.json";

const umi = createUmi(
  process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
);

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(
  irysUploader({
    address: "https://devnet.irys.xyz/",
  }),
);

umi.use(signerIdentity(signer));

(async () => {
  try {
    //chanege image path to your image path
    const image = await readFile("./image/58167D8A-157A-4234-9D78-67797D9754F0_1_105_c.jpeg");

    //change the image name and mime type
    const file = createGenericFile(image, "58167D8A-157A-4234-9D78-67797D9754F0_1_105_c.jpeg")

    const [myUri] = await umi.uploader.upload([file])
    console.log("Your image URI: ", myUri);
  } catch (error) {
    console.log(error);
  }
})();
