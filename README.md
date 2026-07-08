# ISET eTR on Stellar

**Electronic Promissory Notes (ePN) — issued as legally-structured digital records, post-quantum signed, settling in Stellar USDC.**

Live demo: **https://stellar.iset.finance** · Registry & signing engine: [iset.finance](https://iset.finance)

Built for the **APAC Stellar Hackathon** and the **Stellar Journey to Mastery** builder program.

## What it does

- **Issue an ePN** — maker (promisor) + TIN, payee + TIN, principal, interest rate, maturity, place of issue (Philippine legal structure). One click issues a **sandbox** record on the live ISET registry: ML-DSA-65 (post-quantum) signed, hash-chained, court-grade audit trail.
- **Verify it** — every note carries a QR + VERIFY link; anyone can re-check the record against the registry by reference. No account needed.
- **Stellar wallet** — connect Freighter (testnet); the connected public key is bound into the issued record (`window.STELLAR_PUBKEY`).
- **Soroban** — deployed testnet contract: `CA66QGKVHDBFYUUN5LBH4ELANRVWOFHX5QJJIPCW7JUNSOO6CAHQE3KV` ([Stellar Expert](https://stellar.expert/explorer/testnet/contract/CA66QGKVHDBFYUUN5LBH4ELANRVWOFHX5QJJIPCW7JUNSOO6CAHQE3KV)).

## Why it matters

$11T of trade still runs on paper. Under **UNCITRAL MLETR**, an electronic transferable record needs one identifiable controller — the digital equivalent of *holding* the paper. ISET eTR makes that real: the record **is** the instrument, whoever holds it controls it, and it settles in any currency. This demo brings that rail to Stellar — USDC settlement for real-world credit instruments.

## Stack

Static HTML/CSS/vanilla JS — no framework, no build step. Freighter via CDN. Issuance and verification call the production ISET engine (sandbox scope). Hosted on Cloudflare Pages.

## Run locally

Open `index.html` in a browser, or serve the folder with any static server. That's it.

## Honesty notes

- Records issued here are **sandbox**: fully signed, fully verifiable, **no legal effect**.
- The Soroban contract is displayed to prove the chain link; on-chain anchoring from this UI is the next milestone, not a shipped claim.

## License

MIT — see [LICENSE](LICENSE).

---

© 2026 ISET. `fiduciary@iset.finance`
