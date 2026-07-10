##### **ISET eTR: The Neutral Trust Layer for Stellar RWA**

APAC STELLAR HACKATHON · LOCAL FINANCE \& RWA TRACK



Bridging private legal control with public blockchain settlement. ISET decouples the legal state of trade documents from the payment rail, allowing institutions to maintain privacy while leveraging Stellar for automated settlement.



**📌 Deployed Soroban Contract Addresses**

Mainnet: `CBCKBID6DS4MAOSUAUHAQQ7HMD4ONJQD7WA4HPSWZSAVXUVM7LRQKWEH`
(https://stellar.expert/explorer/public/contract/CBCKBID6DS4MAOSUAUHAQQ7HMD4ONJQD7WA4HPSWZSAVXUVM7LRQKWEH)

Testnet: `CA66QGKVHDBFYUUN5LBH4ELANRVWOFHX5QJJIPCW7JUNSOO6CAHQE3KV`
(https://stellar.expert/explorer/testnet/contract/CA66QGKVHDBFYUUN5LBH4ELANRVWOFHX5QJJIPCW7JUNSOO6CAHQE3KV)



**The Problem: The RWA Trust Gap**
Institutions refuse to put commercial trade data onto public blockchains due to privacy laws and competitive secrecy. Yet, without on-chain verification, RWA (Real World Asset) settlements cannot be automated.



**The Solution: The Settlement Router**
ISET eTR acts as a Neutral Trust Layer. We separate the Legal State (managed by ISET off-chain) from the Settlement State (managed by Stellar on-chain).

Off-Chain (ISET): Holds the legal state of the document, fully compliant with international standards.
The Bridge: When legal ownership changes off-chain, ISET generates a Post-Quantum cryptographic proof.
On-Chain (Stellar): The proof triggers a Soroban Smart Contract to execute USDC settlement on Stellar. Stellar provides the liquidity rail; ISET provides the legal truth.



**The ISET eTR Suite (The 6 Instruments)**
While this hackathon demo focuses on a single use case, the ISET engine supports the full lifecycle of global trade:

ePN (Electronic Promissory Note)
eBE (Electronic Bill of Exchange)
eBL (Electronic Bill of Lading)
eLC (Electronic Letter of Credit)
eWR (Electronic Warehouse Receipt)
eINV (Electronic Invoice)



**🚀 Hackathon Demo: ePN Factoring (Proof of Life)**
To prove the architecture works, we deployed a functional Soroban smart contract simulating an ePN (Promissory Note) Factoring flow for MSMEs:

init: An MSME buyer issues an ePN. The maker, lender, amount, and maturity are locked into the Soroban contract.
release\_funds: The MSME seller needs cash today. They legally endorse the ePN to a Lender via ISET. The lender calls this function, passing the ISET legal proof to unlock the funds.
claim\_default: If the ePN reaches maturity and the buyer hasn't paid, the maker can claim the funds back.
If our architecture can securely execute a micro ePN factoring flow, the exact same engine executes a $10M Bill of Lading.



**⚖️ Legal Compliance (Global \& Philippine Framework)**
The ISET eTR engine is built to handle the diverse legal requirements of all 6 trade instruments:

UNCITRAL MLETR: The foundational international standard allowing all 6 instruments to exist as legally enforceable electronic transferable records with a single identifiable controller.
RA 8792 (Electronic Commerce Act): Validates electronic signatures, data messages, and digital documents across all instruments.
Act No. 2031 (Negotiable Instruments Law): Governs the specific legal formatting and transfer of negotiable instruments (ePN, eBE).
Documents of Title \& Warehouse Receipts Laws: Governs the transfer of physical asset rights (eBL, eWR).
UCP 600 \& ISP98: International standard frameworks integrated for Letters of Credit (eLC) and Standby credits.
RA 11057 (PPSR): Perfects security interests for receivables and movable assets (ePN, eINV, eWR) to protect lenders against double-financing.



**🛠️ Tech Stack**
Smart Contracts: Rust / Soroban SDK 27.0.0 (stable) · 6 passing tests (`cargo test`)
Cryptography: Post-Quantum (ML-DSA-65) for off-chain legal signing
Network: Stellar Mainnet (live) + Testnet (free demo)
Architecture: Off-chain legal ledger + On-chain settlement execution

