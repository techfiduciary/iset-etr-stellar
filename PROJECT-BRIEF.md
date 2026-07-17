# ISET eTR on Stellar — Project Brief

*A self-contained reference for taking this project into any conversation, review, or handoff. Written 2026-07-12.*

---

## 1. What this is

**ISET eTR on Stellar** is a live web app that issues **Electronic Transferable Records (eTR)** — legally-structured digital trade instruments — and settles them on the Stellar blockchain. It's the Stellar-network integration of **ISET** (Indigenous Sovereign Estate Trust), whose core product (`iset.finance`) is a production registry that issues six trade instruments under a post-quantum-signed, MLETR-aligned legal framework.

**Live app:** https://stellar.iset.finance
**GitHub:** https://github.com/techfiduciary/iset-etr-stellar (public, MIT license)
**Registry engine:** https://iset.finance (production ISET system; this app calls its sandbox API)

**The one-line pitch:** *A promissory note (or any of six trade instruments) that is signed, verifiable, legally structured — and settles on Stellar in USDC or XLM.*

---

## 2. The core architectural idea

Trade documents have two kinds of state, and this system deliberately keeps them apart:

```
┌─────────────────────────────┐        ┌──────────────────────────────┐
│  ISET Registry (off-chain)  │        │  Stellar / Soroban (on-chain) │
│  · legal state & control    │  proof │  · settlement state           │
│  · ML-DSA-65 signatures     │───────►│  · epn_escrow contract        │
│  · hash-chained audit log   │        │  · USDC / XLM settlement rail │
└─────────────────────────────┘        └────────────────────────────────┘
```

- **Legal state** — who controls the instrument (under UNCITRAL MLETR's "single controller" model), the instrument's terms, and its post-quantum signature — lives in ISET's registry, off-chain. This keeps commercial/legal data private (institutions won't put trade terms on a public chain).
- **Settlement state** — whether payment has moved — is anchored on Stellar via a Soroban smart contract. This gets the speed/cost/finality benefits of a public chain without exposing legal terms.

This is why the architecture is sometimes described as a **"neutral trust layer"** — it bridges private enterprise legal state to public blockchain settlement, rather than putting everything on-chain or everything off-chain.

---

## 3. The six instruments

The app is built around **one data-driven instrument registry** — a single JS config object (`INSTR` in `index.html`) where each instrument defines its own form fields, document template, and status state machine. Adding an instrument is a data change, not new code.

| Code | Name | Legal character | Status |
|---|---|---|---|
| **ePN** | Electronic Promissory Note | Unconditional *promise to pay* | ✅ **Active — the only instrument currently issuable** |
| eBE | Electronic Bill of Exchange | Unconditional *order to pay* (drawer → drawee → payee) | 🔒 Visible, form fields shown but locked (not typeable/clickable) |
| eBL | Electronic Bill of Lading | Receipt for shipped goods + **title document** to the cargo | 🔒 Locked |
| eLC | Electronic Letter of Credit | Bank's conditional payment guarantee (UCP 600) | 🔒 Locked |
| eWR | Electronic Warehouse Receipt | Receipt for stored commodities, usable as collateral | 🔒 Locked |
| eINV | Electronic Invoice | Verifiable demand for payment (not negotiable) | 🔒 Locked |

**Why 5 of 6 are locked:** the hackathon/competition entry is scoped to ePN only. The other five are fully built (real forms, real document templates, real state machines) so the product story is visible, but issuance is intentionally disabled until scope expands — see §9.

### ePN field spec (the active instrument)
Maker + TIN · Payee ("pay to the order of") + TIN · Principal amount · Base currency · Interest rate % p.a. · **Withholding tax %** (sample value `1`) · Issue date · Maturity date · Place of issue · Purpose of note (Working Capital / Trade Finance / Inventory Financing / Equipment Purchase / General Operations) · Settlement currency (USDC or XLM only) · Remarks.

### The other five (fully speced, currently locked)
- **eBE:** Drawer/TIN, Drawee/TIN, Payee/TIN, Amount, Tenor (days to maturity), Place of drawing.
- **eBL:** Shipper/TIN, Carrier, Consignee/TIN, Notify party, Vessel/Voyage no., Port of loading, Port of discharge, Description of goods, eBL clauses (Shipped on Board / Clean on Board / Freight Prepaid checkboxes), Declared cargo value.
- **eLC:** Applicant (Buyer)/TIN, Beneficiary (Seller)/TIN, Issuing bank, Confirming bank, Credit amount, Expiry date, Latest shipment date, Required-documents checklist (Commercial Invoice, Transport Docs, Insurance Certificate, Packing List, Certificate of Origin), UCP 600 reference.
- **eWR:** Depositor/TIN, Warehouse operator/TIN, Warehouse license number, Commodity type, Quality/grade, Quantity/weight, Storage location, Deposit date, Claimable-until date.
- **eINV:** Seller/TIN, Buyer/TIN, Invoice number, Date of issue, Due date, Payment terms, Line item description, Quantity × Unit price, VAT %, Withholding tax % — computes Subtotal → VAT → WHT → Total payable live.

---

## 4. Enterprise layers (what makes it more than a form)

1. **Status state machine.** Every document carries a persistent status badge. ePN: `DRAFT → ISSUED → ENDORSED → SETTLED → MATURED`. Each instrument has its own sequence (e.g. eBL: `DRAFT → ISSUED → LOADED ON VESSEL → ARRIVED AT PORT → TITLE TRANSFERRED`). Action buttons (Endorse/Settle) are disabled unless the document is in the correct state; the badge advances automatically as on-chain confirmations land.
2. **Hash-chained audit trail.** Every create/endorse/settle/amend action writes a timestamped entry with the **real SHA-256** hash of the previous state and the new state (`crypto.subtle.digest`, never fabricated). Shown per-document and as a global feed on the Dashboard. This is the "prove an immutable paper trail" story for institutional reviewers.
3. **Identity badge.** Each document shows *"Issuer verified via ISET registry · TIN on file · personal data stays off-chain"* — honest wording: it reflects that a TIN is on file in the registry, not a KYC claim the app doesn't actually perform.
4. **Cross-border FX & withholding tax.** Settlement Currency is restricted to **USDC — Stellar USDC** or **XLM — Stellar Lumen** (both real Stellar settlement assets). When the settlement currency differs from the document's base currency, an indicative FX-rate field appears and computes an estimated settlement amount. The ePN's Withholding Tax field computes and deducts tax, showing **Principal → Withholding (less) → Net proceeds** in a Financials section — while the document's headline amount stays the **face value** (the principal actually promised), not the net, matching the legal wording above it.
5. **Amendment workflow.** An *Amend* button is enabled only while a document is `ISSUED`. Amending locks the current version into the audit trail as `AMENDED` and spawns a new version (`id-v2`, fresh audit trail, `amends: <old-id>` link back).
6. **Dashboard.** Portfolio stats (document count, total value by currency, completed count, live wallet-connection status), the six instruments as clickable cards, a global audit feed, and the on-chain contract info — all in one view, not scattered across screens.

---

## 5. On-chain (Stellar / Soroban)

**Contract:** `epn_escrow` — Rust, Soroban SDK 27.0.0 (stable). Source: [`contracts/epn_escrow/`](https://github.com/techfiduciary/iset-etr-stellar/tree/main/contracts/epn_escrow) in the repo.

| Network | Contract ID | Explorer |
|---|---|---|
| **Mainnet** (live) | `CBDFHJ2V7YBQGF4PNDPTKAALHIE6PYRBL6PVAGMJA5OFFEUMHJYQLL7O` | [Stellar Expert →](https://stellar.expert/explorer/public/contract/CBDFHJ2V7YBQGF4PNDPTKAALHIE6PYRBL6PVAGMJA5OFFEUMHJYQLL7O) |
| **Testnet** (free demo) | `CDTZDPLIB7OZ5LTCXAEY4GWUQTDKVGEGSSUWRF56MUBDNE5DSNSMXEGH` | [Stellar Expert →](https://stellar.expert/explorer/testnet/contract/CDTZDPLIB7OZ5LTCXAEY4GWUQTDKVGEGSSUWRF56MUBDNE5DSNSMXEGH) |

Both networks run the **identical wasm** (hash `7fb80059088d523730858e6ab8b90c5de8066b4bb369672c87d514cc36e695c2`), built and tested by CI on every push.

**Mainnet deployment transactions (real XLM spent):**
- WASM upload: [`27dc4c53…`](https://stellar.expert/explorer/public/tx/27dc4c53f19f4c77ebb06a45ab013d663922f6abd00a98ea2d61ca82a5de7a7d) — cost ~7.6 XLM
- Contract deploy: [`5605a984…`](https://stellar.expert/explorer/public/tx/5605a984b7bc19ade6d3111b04ec393b4a55c8cedfbf2f9d46b71fb48af6d7e4)

### Contract interface
```rust
fn init(maker: Address, lender: Address, amount: i128, maturity: u64, iset_pubkey: BytesN<32>)
fn release_funds(caller: Address, iset_signature: BytesN<64>)
fn claim_default(caller: Address)
fn status() -> Status        // Active | Released | Defaulted
fn terms() -> Terms          // { maker, lender, amount, maturity }
```
Typed `#[contracterror] Error` enum (`InvalidAmount`, `NotInitialized`, `NotActive`, `Unauthorized`, `NotMatured`), events published on every state transition, doc comments explaining the off-chain/on-chain trust split. 8 passing tests (`cargo test`) covering settle, double-settle rejection, access control, zero-amount rejection, and both maturity paths.

**Demo flow model:** the connected wallet is registered as *both* maker and lender at `init()` — this lets a single reviewer run the full issue → endorse → settle lifecycle solo, with real transactions and real hashes, rather than needing two separate wallets.

**On-chain settlement is currently ePN-only** (the deployed contract is `epn_escrow`, specific to that instrument). The other five instruments issue and verify on the ISET registry and advance their status **off-chain**, with every move recorded in the same hash-chained audit trail — this is disclosed in the app, not hidden.

### CI (why it matters)
`.github/workflows/contract.yml` runs `cargo test` + `cargo fmt --check` + a wasm release build on every push. This exists because the local dev machine (7 GB RAM) reliably OOM-crashes `rustc` (`STATUS_STACK_BUFFER_OVERRUN`) when compiling the full native test harness — GitHub's runners don't have that constraint. Local wasm builds still work with `CARGO_BUILD_JOBS=1`.

---

## 6. Wallets & networks

- **Wallets:** [Freighter](https://freighter.app) (browser extension, tried first) → falls back to [Albedo](https://albedo.link) (web-based, works on mobile too, no install needed). Connected pubkey is bound into every issued record.
- **Network toggle:** Testnet/Mainnet switch lives in the top banner (visible on every tab) and on the Dashboard. **Testnet is the default** — free, auto-funded via friendbot, zero real cost, so any reviewer can run the whole flow at no expense. Switching to Mainnet is confirm-gated ("this will spend real XLM") since it submits real transactions.
- **Mainnet activation:** a wallet needs an existing account (funded with ≥1 XLM) to transact on mainnet — Stellar mainnet has no free faucet. If a connected wallet has no mainnet account yet, the app shows a self-service funding card (address + copy button + a link to check funding status) instead of a dead-end error.

---

## 7. App structure

Single HTML file, **no framework, no build step** — `index.html` is the entire frontend (~1,000 lines: styles, markup, and vanilla JS in one `<script>`).

- **Views:** Dashboard · Issue · Documents · Verify · Brand (identity/branding settings) — tab-based, `hidden` attribute toggling, no router.
- **State:** everything persists in `localStorage` per-device (`epn-notes`, `epn-brand`, `epn-net`, `epn-theme`, `epn-wallet`) — no server-side user accounts.
- **Registry client:** calls the production ISET engine at `https://iset.finance/api/etr/stellar-issue` (issuance) and `/api/etr/verify/:id` (verification).
- **Responsive:** mobile-first with a wider desktop layout (≥900px: 1160px max-width, 2-column dashboard, wider forms) — same features and sections at both sizes.
- **Theme:** light/dark toggle, persisted.

### Backend integration (in the ISET production worker, `02-ISET-finance/dist/_worker.js`)
A dedicated public route, `POST /api/etr/stellar-issue`, exists specifically for this app: no session/login required (issuance is normally session-gated, but Stellar wallets don't do the site's EVM-style sign-in). The subject is derived from the caller's Stellar pubkey (`stellar:<pubkey>`), which has no production issuer agreement — so the engine automatically flags every record **sandbox** (fully signed and verifiable, but explicitly no legal effect). The route validates `type` against an allow-list of all six instrument codes and is capped at 6KB per request. CORS is scoped to `stellar.iset.finance` and its preview subdomains only.

---

## 8. Deployment

- **Hosting:** Cloudflare Pages, project `iset-etr-stellar`.
- **Gotcha:** always `rm -rf contracts/epn_escrow/target` before `wrangler pages deploy` — Rust debug artifacts (`.pdb` files) exceed Cloudflare Pages' 25 MiB per-file limit and will hard-fail the deploy.
- **Deploy command:** `npx wrangler pages deploy . --project-name=iset-etr-stellar --branch=main` (from the repo root, with `CLOUDFLARE_API_TOKEN`/`CLOUDFLARE_API_KEY` unset so it uses OAuth).

---

## 9. Known limitations (stated honestly, not hidden)

- Records issued through this app are **sandbox**: fully signed and independently verifiable, but carry **no legal effect** — this is disclosed in the UI banner and footer.
- **5 of 6 instruments are feature-complete but issuance-locked** — the competition entry is scoped to ePN; the rest are a visible roadmap, not vaporware (real forms, real document templates already exist in the code).
- **On-chain settlement is ePN-only** — no Soroban escrow exists yet for the other five instruments.
- MLETR is cited as the **design standard the records are aligned to**, not a claim of enacted law in any specific jurisdiction (the Philippines has not enacted MLETR).
- No formal third-party smart-contract security audit yet (roadmap item).

---

## 10. Roadmap

| When | Milestone |
|---|---|
| **Q3 2026** (shipped) | ePN factoring flow live end-to-end — app + registry + Soroban contract on testnet and mainnet, CI-verified tests, RA 8792-structured records |
| **Q4 2026** | Full 6-instrument suite wired to on-chain settlement; lender-partner onboarding (PH pilot cohort) |
| **Q1 2027** | Formal smart-contract security audit; 10 institutional pilot users on mainnet |
| **Q2 2027+** | Scale past 100 users; Stellar Community Fund (SCF) grant application; multi-corridor settlement |

---

## 11. Team

**Francis Neri — Founder & Tech Fiduciary.** Trade-finance infrastructure architect. ISET operates a live production registry at [iset.finance](https://iset.finance); this Stellar app is its blockchain settlement integration. Contact: fiduciary@iset.finance.

---

## 12. Recent notable fixes (engineering changelog highlights)

- **Mainnet deploy fee off-by-one:** Soroban's `minResourceFee` from simulation is a *lower bound*, not the final required fee — a bare default landed exactly 1 stroop short and bounced as `txINSUFFICIENT_FEE`. Fixed with an explicit fee buffer.
- **Dashboard "Wallet" stat bug:** the Portfolio stat card checked a persisted `localStorage` flag instead of the live `window.STELLAR_PUBKEY`, so it could show "Connected" after a disconnect or on a fresh load where a stale flag lingered (e.g. Albedo doesn't auto-reconnect on reload). Fixed to check live state and refresh immediately on both connect and disconnect, with no tab navigation required.
- **Contract professionalization:** rewrote the Soroban contract from a minimal hackathon version (unlabeled state, no events, `.expect()`-style panics) to typed errors, a named `Status` state machine, emitted events, and doc comments — then redeployed to both networks with matching wasm, verified by CI.

---

*Everything above reflects the live state of stellar.iset.finance and github.com/techfiduciary/iset-etr-stellar as of this writing. For the freshest detail, check the repo directly — this file is a snapshot for portable discussion, not the source of truth.*
