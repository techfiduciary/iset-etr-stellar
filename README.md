# ISET eTR on Stellar — ePN

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

Soroban escrow contract (Stellar testnet): `CA66QGKVHDBFYUUN5LBH4ELANRVWOFHX5QJJIPCW7JUNSOO6CAHQE3KV` ([Stellar Expert](https://stellar.expert/explorer/testnet/contract/CA66QGKVHDBFYUUN5LBH4ELANRVWOFHX5QJJIPCW7JUNSOO6CAHQE3KV))

Contract source: [`contracts/epn_escrow/`](contracts/epn_escrow/) — Rust / Soroban SDK, with tests and Makefile.

## Why

$11T of trade still runs on paper. Under **UNCITRAL MLETR**, an electronic transferable record needs one identifiable controller — the digital equivalent of *holding* the paper. ISET eTR makes that real: the record **is** the instrument, whoever holds it controls it, and it settles in any currency. This app brings that rail to Stellar — USDC settlement for real-world credit instruments.

## Stack

Static HTML/CSS/vanilla JS — no framework, no build step. Freighter via CDN. Issuance and verification call the production ISET engine (sandbox scope). Soroban contract in Rust. Hosted on Cloudflare Pages.

## Run locally

Serve the folder with any static server (or open `index.html`). Contract: `cd contracts/epn_escrow && make test`.

## Notes

Records issued here are **sandbox**: fully signed, fully verifiable, **no legal effect**.

## License

MIT — see [LICENSE](LICENSE).

---

© 2026 ISET · fiduciary@iset.finance
