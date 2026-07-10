# ISET eTR

**Electronic Promissory Notes — issued as legally-structured digital records, post-quantum signed, settling in Stellar USDC.**

Live: **https://stellar.iset.finance** · Registry & signing engine: [iset.finance](https://iset.finance)

## What it does

- **Issue an ePN** — maker (promisor) + TIN, "pay to the order of" payee + TIN, principal (PHP default), interest, maturity, place of issue. One tap issues a **sandbox** record on the live ISET registry: ML-DSA-65 (post-quantum) signed, hash-chained, auditable.
- **Notes** — your issued notes on this device; open any note as a signed B2B document with QR + VERIFY.
- **Verify** — anyone can re-check a record against the registry by reference; no account needed.
- **Connect** — Freighter wallet (Stellar testnet); the connected public key is bound into every note you issue.
- **Brand** — your company name and color, applied to the note header.
- Light/dark themes, mobile-first, print-to-PDF.

## On-chain

The `epn_escrow` Soroban contract is **deployed and live on Stellar mainnet**.

| Network | Contract ID | Explorer |
|---|---|---|
| **Mainnet** | `CBCKBID6DS4MAOSUAUHAQQ7HMD4ONJQD7WA4HPSWZSAVXUVM7LRQKWEH` | [Stellar Expert](https://stellar.expert/explorer/public/contract/CBCKBID6DS4MAOSUAUHAQQ7HMD4ONJQD7WA4HPSWZSAVXUVM7LRQKWEH) |
| Testnet | `CA66QGKVHDBFYUUN5LBH4ELANRVWOFHX5QJJIPCW7JUNSOO6CAHQE3KV` | [Stellar Expert](https://stellar.expert/explorer/testnet/contract/CA66QGKVHDBFYUUN5LBH4ELANRVWOFHX5QJJIPCW7JUNSOO6CAHQE3KV) |

Mainnet deployment transactions:
- WASM upload — [`99dcb16b…`](https://stellar.expert/explorer/public/tx/99dcb16b03af2c5f0d3118c369ba215ad70c25292b862c04329556777bb4cdca) (wasm hash `109d5ce873ef7721664717ebb1db1e27ad56cc75f97e3a57372618118c4e9f8a`)
- Contract deploy — [`a570c1d0…`](https://stellar.expert/explorer/public/tx/a570c1d037a1f70693131eb46922a5cef4e0a2554b7ca58f7a10c313f2b4f2a0)

The app defaults to testnet (free, auto-funded) so anyone can run the full flow at no cost; switch to mainnet from the **On-chain** section on Home, or with `?net=mainnet`.

Contract source: [`contracts/epn_escrow/`](contracts/epn_escrow/) — Rust / Soroban SDK 27, `cargo test` → 6 passing tests covering settle, access control, zero-amount, and maturity paths.

## Why

$11T of trade still runs on paper. Under **UNCITRAL MLETR**, an electronic transferable record needs one identifiable controller — the digital equivalent of *holding* the paper. ISET eTR makes that real: the record **is** the instrument, whoever holds it controls it, and it settles in any currency. This app brings that rail to Stellar — USDC settlement for real-world credit instruments.

## Stack

Static HTML/CSS/vanilla JS — no framework, no build step. Freighter via CDN. Issuance and verification call the production ISET engine (sandbox scope). Soroban contract in Rust. Hosted on Cloudflare Pages.

## Run locally

Serve the folder with any static server (or open `index.html`). Contract: `cd contracts/epn_escrow && cargo test`.

Note: run `rm -rf contracts/epn_escrow/target` before `wrangler pages deploy` — Rust debug artifacts exceed Pages' 25 MiB per-file limit.

## Notes

Records issued here are **sandbox**: fully signed, fully verifiable, **no legal effect**.

## License

MIT — see [LICENSE](LICENSE).

---

© 2026 ISET · fiduciary@iset.finance
