#![no_std]

//! # ePN Escrow — ISET eTR settlement contract
//!
//! Settlement state machine for a single Electronic Promissory Note (ePN).
//!
//! ## Trust model
//! The legal state of the note — who controls it under UNCITRAL MLETR's
//! single-controller model — is held off-chain in the ISET registry and signed
//! with post-quantum cryptography (ML-DSA-65). This contract holds only the
//! *settlement* state and gates it on the lender's on-chain authorization
//! (`require_auth`). The ISET public key and its signature are recorded on-chain
//! (in storage and events) as a tamper-evident audit anchor linking the
//! settlement to the off-chain legal proof.
//!
//! On-chain verification of the ML-DSA-65 proof is a roadmap item, pending
//! Soroban host-function support for post-quantum signature schemes; today the
//! registry performs that verification off-chain before authorizing the lender.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    BytesN, Env,
};

/// Errors surfaced to callers as typed contract errors.
#[contracterror]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Error {
    /// The principal amount must be greater than zero.
    InvalidAmount = 1,
    /// The escrow has not been initialized with a note.
    NotInitialized = 2,
    /// The note is not in the `Active` state (already released or defaulted).
    NotActive = 3,
    /// The caller is not the party authorized for this action.
    Unauthorized = 4,
    /// The note has not yet reached its maturity date.
    NotMatured = 5,
}

/// Lifecycle state of the note.
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    /// Issued and outstanding.
    Active,
    /// Endorsed to and settled by the lender.
    Released,
    /// Reclaimed by the maker after maturity passed unpaid.
    Defaulted,
}

/// The note's economic and party terms, set at initialization.
#[contracttype]
#[derive(Clone)]
pub struct Terms {
    /// The maker (promisor) who unconditionally promises to pay.
    pub maker: Address,
    /// The payee/lender the note is payable to.
    pub lender: Address,
    /// Principal amount, in the note's currency (minor units).
    pub amount: i128,
    /// Maturity as a Unix timestamp (seconds).
    pub maturity: u64,
}

/// Instance-storage keys.
#[contracttype]
enum DataKey {
    /// The `Terms` of the note.
    Terms,
    /// The current `Status`.
    Status,
    /// The 32-byte ISET public key anchoring the off-chain legal proof.
    IsetKey,
}

#[contract]
pub struct EpnEscrow;

#[contractimpl]
impl EpnEscrow {
    /// Initialize the escrow for one ePN.
    ///
    /// Records the note's terms, anchors the ISET public key, and sets the note
    /// `Active`. Must be authorized by the `maker`.
    ///
    /// # Panics
    /// [`Error::InvalidAmount`] if `amount <= 0`.
    pub fn init(
        e: Env,
        maker: Address,
        lender: Address,
        amount: i128,
        maturity: u64,
        iset_pubkey: BytesN<32>,
    ) {
        if amount <= 0 {
            panic_with_error!(&e, Error::InvalidAmount);
        }
        maker.require_auth();

        let terms = Terms {
            maker: maker.clone(),
            lender: lender.clone(),
            amount,
            maturity,
        };
        let store = e.storage().instance();
        store.set(&DataKey::Terms, &terms);
        store.set(&DataKey::IsetKey, &iset_pubkey);
        store.set(&DataKey::Status, &Status::Active);

        e.events()
            .publish((symbol_short!("init"),), (maker, lender, amount, maturity));
    }

    /// Endorse and settle the note to the lender.
    ///
    /// Only the registered lender may call this, and only while the note is
    /// `Active`. The `iset_signature` — ISET's proof that the off-chain legal
    /// endorsement is valid — is recorded in the `released` event as an audit
    /// anchor. Transitions the note to `Released`.
    ///
    /// # Panics
    /// [`Error::Unauthorized`] if the caller is not the lender;
    /// [`Error::NotActive`] if the note is not `Active`.
    pub fn release_funds(e: Env, caller: Address, iset_signature: BytesN<64>) {
        let terms = Self::read_terms(&e);
        if caller != terms.lender {
            panic_with_error!(&e, Error::Unauthorized);
        }
        caller.require_auth();
        Self::require_active(&e);

        e.storage()
            .instance()
            .set(&DataKey::Status, &Status::Released);
        e.events()
            .publish((symbol_short!("released"),), (caller, iset_signature));
    }

    /// Reclaim the note after maturity if it was never settled.
    ///
    /// Only the maker may call this, only while the note is `Active`, and only
    /// once the maturity timestamp has passed. Transitions to `Defaulted`.
    ///
    /// # Panics
    /// [`Error::Unauthorized`] if the caller is not the maker;
    /// [`Error::NotActive`] if the note is not `Active`;
    /// [`Error::NotMatured`] if maturity has not yet passed.
    pub fn claim_default(e: Env, caller: Address) {
        let terms = Self::read_terms(&e);
        if caller != terms.maker {
            panic_with_error!(&e, Error::Unauthorized);
        }
        caller.require_auth();
        Self::require_active(&e);

        if e.ledger().timestamp() <= terms.maturity {
            panic_with_error!(&e, Error::NotMatured);
        }

        e.storage()
            .instance()
            .set(&DataKey::Status, &Status::Defaulted);
        e.events()
            .publish((symbol_short!("defaulted"),), (caller, terms.maturity));
    }

    /// Read the current settlement status.
    pub fn status(e: Env) -> Status {
        e.storage()
            .instance()
            .get(&DataKey::Status)
            .unwrap_or_else(|| panic_with_error!(&e, Error::NotInitialized))
    }

    /// Read the note's terms.
    pub fn terms(e: Env) -> Terms {
        Self::read_terms(&e)
    }

    // ── internal ────────────────────────────────────────────────────────────

    fn read_terms(e: &Env) -> Terms {
        e.storage()
            .instance()
            .get(&DataKey::Terms)
            .unwrap_or_else(|| panic_with_error!(e, Error::NotInitialized))
    }

    fn require_active(e: &Env) {
        let status: Status = e
            .storage()
            .instance()
            .get(&DataKey::Status)
            .unwrap_or_else(|| panic_with_error!(e, Error::NotInitialized));
        if status != Status::Active {
            panic_with_error!(e, Error::NotActive);
        }
    }
}

mod test;
