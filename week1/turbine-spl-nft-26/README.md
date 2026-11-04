# scripts-solana

Scripts for creating SPL tokens and NFTs on Solana devnet.

---

## Setup

### 1. Add your wallet

Place your devnet wallet keypair file at the project root:

```
root/
└── devnet-wallet.json   ← here
```

It should be a JSON array of numbers, e.g. `[174, 23, ...]`.

### 2. Install dependencies

```bash
npm install
```

```bash
npm install --save-dev @types/node ts-node typescript
```

### 3. Add your image

Place your image at the project root.

```
root/
└── image.jpeg   ← here
```

---

> Before running the scripts, go through these docs:
> - [Solana token docs](https://solana.com/docs/tokens) — mint accounts, token accounts, and ATAs
> - [Solana Kit](https://www.solanakit.com/) — the JS SDK used for building and sending transactions
> - [Metaplex Token Metadata](https://www.metaplex.com/docs/smart-contracts/token-metadata) — attaching metadata to SPL tokens
> - [Metaplex Core](https://www.metaplex.com/docs/smart-contracts/core) — the NFT standard used in the NFT scripts

## SPL Token

Uses **@solana/kit** and **@solana-program/token** for transactions, and **mpl-token-metadata** via UMI for on-chain metadata.

| Script | Command | What it does |
|---|---|---|
| `spl_init.ts` | `npm run spl:init` | Creates a new mint account |
| `spl_metadata.ts` | `npm run spl:metadata` | Attaches a name, symbol, and URI to the mint |
| `spl_mint.ts` | `npm run spl:mint` | Creates your associated token account and mints tokens into it |
| `spl_transfer.ts` | `npm run spl:transfer` | Sends tokens to another wallet i.e ata to ata |

Run them in order. Each script logs the addresses/signatures you'll need to paste into the next one.

---

## NFT

Uses **@solana/kit** and **mpl-core** via UMI. Images and metadata are stored on Irys (decentralized storage).

| Script | Command | What it does |
|---|---|---|
| `nft_image.ts` | `npm run nft:image` | Uploads your image to Irys, logs the image URI |
| `nft_metadata.ts` | `npm run nft:metadata` | Builds the metadata JSON and uploads it, logs the metadata URI |
| `nft_mint.ts` | `npm run nft:mint` | Mints the NFT on-chain using the metadata URI |
| `nft_transfer.ts` | `npm run nft:transfer` | Transfers ownership of an NFT you own to another wallet |
| `nft_burn.ts` | `npm run nft:burn` | Burns an NFT you own and reclaims its rent |
| `nft_update.ts` | `npm run nft:update` | Updates the name and URI of an NFT you are the update authority of |
| `nft_seller_update.ts` | `npm run nft:seller-update` | Demo: mints, sells to a fresh wallet, then the seller edits the metadata the buyer now owns |

Run them in order. Paste the URI logged by each step into the next script before running it.

---

## Week 1 record

All transactions ran on **devnet** from wallet [`3Vm8KKkd…`](https://explorer.solana.com/address/3Vm8KKkd1aaCL1nnVf4PSXMZySy7Cu2DrsNVQeGpN2An?cluster=devnet). Each link opens the transaction on Solana Explorer.

### SPL token

| Task | Proof | Notes |
|---|---|---|
| Create mint (6 decimals) | [`5KbW2ars…`](https://explorer.solana.com/tx/5KbW2arsgy7xNK5mz2usnymz8Bo9o31frwKywEgLWN9KANBtYuMgUSp9PB7MxfPpo8QKnfo7ZxAYyvEMawbZjuJC?cluster=devnet) | Mint [`AmKWggCE…`](https://explorer.solana.com/address/AmKWggCEt3oviZNxkyJi5oAyiKovzyLMMnWjXBFVWuW6?cluster=devnet) |
| Attach metadata (Turbin3 Token, TRB3) | [`4vxVVWE2…`](https://explorer.solana.com/tx/4vxVVWE2rRfgQDGNpCyroFtqhHt7va759yY8dEyd7inE3sAsApLLzGP9LW7hznRqKdbmAvtd5MkKpugFrw7ryE8N?cluster=devnet) | Metaplex Token Metadata |
| Create ATA and mint 1,000 tokens | [`4PVHAsaY…`](https://explorer.solana.com/tx/4PVHAsaY5VMjZkps55kBkJH2tDtLjnH22jbvYe4hZor8yMZUyjW57MqRNm8Lz5NNDihjnmiHAAS4SShsrBthEnUs?cluster=devnet) | ATA [`GNt5y4CG…`](https://explorer.solana.com/address/GNt5y4CGC136QL6j1uYqW3qYhNktKQ4sQBiU22esWHV7?cluster=devnet) |
| Transfer 100 tokens, ATA to ATA | [`2D9fGjQi…`](https://explorer.solana.com/tx/2D9fGjQifxfbREpNonaLRaTNfEVqFV6aD7vDPd1SXK525VPDumi8aYjyz1HDbyfqFsCYxQKcH2VoPXwoYdpWUQ9R?cluster=devnet) | Recipient ATA [`F3M3eDv8…`](https://explorer.solana.com/address/F3M3eDv8tUVLW8fKEdJQRTNJWcUdaxGkoWuY5po3o599?cluster=devnet) |

### NFT

| Task | Proof | Notes |
|---|---|---|
| Upload image to Irys | [`H49euxJ2…`](https://gateway.irys.xyz/H49euxJ2Viyqo3cg1NLaLLVHuwvUXqraMX59PC22qUYZ) | Irys upload, not a Solana tx |
| Upload metadata JSON to Irys | [`EUU2NT4B…`](https://gateway.irys.xyz/EUU2NT4BTj6k26g3k3gVKkYQ1ZN5P1ZtsGKU5yTjV5AS) | Irys upload, not a Solana tx |
| Mint Core NFT | [`51PvSKv3…`](https://explorer.solana.com/tx/51PvSKv37jfxFk8u8QNk9ReDhWNgVhEKDrYNfBumPfizowGpM3V7HQA5cwSLXxyAf6weiCoP6MUDAXHn5bC7UhZn?cluster=devnet) | Asset [`kARw6p4Z…`](https://explorer.solana.com/address/kARw6p4ZnVXz4SyjrzRF9azFCcT4Mks3Ym1YSLv7CBX?cluster=devnet) |
| Transfer NFT to another wallet | [`b75iYx9a…`](https://explorer.solana.com/tx/b75iYx9aJ5YidjoJ5Re2gFaxsFmxBkbMby7JHJMy8dschUyR5sq7PtiQMJpvBZ9CE8qNfMvMSsgbqVuWuji3LSN?cluster=devnet) | Same asset, new owner [`9EUd4VNc…`](https://explorer.solana.com/address/9EUd4VNcjMAysd7zQk3Q1a4tb28BYndLNBAQDiYnHJ64?cluster=devnet) |
| Update NFT metadata | [`3RbK4YT4…`](https://explorer.solana.com/tx/3RbK4YT4pmmzWGtBiMEiFJ3G52u1XMYH9qzxEMCgnTra5qRRrKsyqLoK1YfWLK4jbLipoqXthqS1rp1AKe9Uo4Zw?cluster=devnet) | Renamed to "swinecoder v2" after transfer |
| Burn NFT and reclaim rent | [`5976nhkn…`](https://explorer.solana.com/tx/5976nhknNUDq59FX35nc8onmq63wvfQr91jqWVtVPZgJCiMfunBGHET5NEpyghzXUkjrcQFvTeRvaZcLpifQYgDD?cluster=devnet) | Asset [`JA76n67y…`](https://explorer.solana.com/address/JA76n67yJDT8gKbnvJdxJAijh1sRSehn8pL2gvXvULDZ?cluster=devnet) |
| Seller edits NFT after sale (demo) | [`65KT3SiS…`](https://explorer.solana.com/tx/65KT3SiSdEyHDFMRa1Nq9biFWF2GGWR7DSn5kPaVDpRSRG3W7LJHnoenM1rhSDFkDLttjSoGWU92iLZTSubLPrDH?cluster=devnet) | Asset [`9SUJ9jAw…`](https://explorer.solana.com/address/9SUJ9jAwK6r9UGbWqVotFMWFr7D2scfZiqVg8eYLK8gp?cluster=devnet), owned by buyer |

### What I learnt

- **Owner and update authority are separate roles in Metaplex Core.** Transferring an NFT moves ownership only, so the creator can still edit its name and URI after a sale. The buyer cannot. `nft:seller-update` proves this end to end. For real-world assets (RWAs) this is useful when facts change, but buyers have to trust whoever holds the update key. Setting the update authority to `None` locks the metadata for good.
- **Burning a Core NFT doesn't return all the rent.** The account shrinks to a 1-byte tombstone so the address can never be reused. Out of 0.00293256 SOL, 0.00077724 SOL came back. The rest is rent for the tombstone plus Metaplex's 0.0015 SOL create fee, which stays in the account.

---

## What changed since Q2 2025

- **@solana/kit replaced web3.js v1 as the recommended SDK.** Kit started life as web3.js 2.0 and is now on major version 8. Solana's docs say new apps should build on Kit directly. A web3.js v3 compatibility layer built on Kit exists for older codebases. ([solana.com](https://solana.com/docs/frontend/web3-compat))
- **Program clients are generated packages.** Instructions come from `@solana-program/token` and `@solana-program/system` instead of being hand-built.
- **Rent is getting cheaper.** SIMD-0437 cuts the rent constant from 6,960 to 696 lamports per byte in five feature-gated steps. Devnet already charges 5,080 lamports per byte, measured while burning an NFT here. That makes mints, ATAs and NFTs about 27% cheaper to create than in 2025. ([solana.com](https://solana.com/upgrades/reduced-rent), [Solana Compass](https://solanacompass.com/news/simd-0437-step-2-goes-live-on-solana-mainnet-rent-drops-to-5080-lamports-per-byte))
- **Metaplex Core is the default NFT standard, and it charges protocol fees.** Creating a Core asset costs 0.0015 SOL on top of rent. ([Metaplex protocol fees](https://developers.metaplex.com/protocol-fees))
- **Bubblegum V2 shipped on May 29, 2025.** Compressed NFTs now work with Core collections and support freeze and thaw, soulbound NFTs and royalty enforcement. ([Metaplex docs](https://developers.metaplex.com/bubblegum-v2))
- **Token-2022 can hold metadata itself.** The metadata pointer and token metadata extensions store a fungible token's name, symbol and URI on the mint, with no separate Metaplex account. ([Solana token extensions](https://solana.com/developers/guides/token-extensions/getting-started))

| Package | Used here | Latest (Nov 2026) |
|---|---|---|
| `@solana/kit` | 6.8.0 | 8.6.0 |
| `@solana-program/token` | 0.13.0 | 0.18.0 |
| `@solana-program/system` | 0.12.0 | 0.16.0 |
| `@metaplex-foundation/mpl-core` | 1.10.0 | 1.10.0 |
| `@metaplex-foundation/umi` | 1.5.1 | 1.6.0 |

