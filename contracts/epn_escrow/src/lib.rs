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
    /// No endorsement offer is outstanding for this note.
    NoPendingEndorsement = 6,
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
    /// The current holder the note is payable to — the original payee at
    /// issuance, updated to the endorsee on each accepted endorsement.
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
    /// The outstanding endorsement offer: the endorsee who may `accept`.
    Pending,
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
        // A fresh note must not inherit a stale endorsement offer from the
        // previous note held in this instance.
        store.remove(&DataKey::Pending);

        e.events()
            .publish((symbol_short!("init"),), (maker, lender, amount, maturity));
    }

    /// Offer to endorse the note to a new holder.
    ///
    /// Only the current holder (`lender`) may endorse, and only while the note
    /// is `Active`. The offer is not effective until the endorsee calls
    /// [`accept`](Self::accept) — endorsement (assignment) plus acceptance are
    /// two distinct authorized acts, mirroring endorsement and delivery of a
    /// paper note. Re-endorsing before acceptance replaces the outstanding
    /// offer (the offer is revocable until accepted).
    ///
    /// # Panics
    /// [`Error::Unauthorized`] if `from` is not the current holder;
    /// [`Error::NotActive`] if the note is not `Active`.
    pub fn endorse(e: Env, from: Address, to: Address) {
        let terms = Self::read_terms(&e);
        if from != terms.lender {
            panic_with_error!(&e, Error::Unauthorized);
        }
        from.require_auth();
        Self::require_active(&e);

        e.storage().instance().set(&DataKey::Pending, &to);
        e.events().publish((symbol_short!("endorse"),), (from, to));
    }

    /// Accept an outstanding endorsement offer and become the note's holder.
    ///
    /// Only the endorsee named by the current holder's [`endorse`](Self::endorse)
    /// may accept, and only while the note is `Active`. On acceptance the
    /// caller becomes the `lender` — the sole party able to settle via
    /// [`release_funds`](Self::release_funds) — and the previous holder loses
    /// all control. Exclusive control transfers atomically; the same note can
    /// never be endorsed to two holders at once.
    ///
    /// # Panics
    /// [`Error::NoPendingEndorsement`] if no offer is outstanding;
    /// [`Error::Unauthorized`] if the caller is not the named endorsee;
    /// [`Error::NotActive`] if the note is not `Active`.
    pub fn accept(e: Env, to: Address) {
        let pending: Address = e
            .storage()
            .instance()
            .get(&DataKey::Pending)
            .unwrap_or_else(|| panic_with_error!(&e, Error::NoPendingEndorsement));
        if to != pending {
            panic_with_error!(&e, Error::Unauthorized);
        }
        to.require_auth();
        Self::require_active(&e);

        let mut terms = Self::read_terms(&e);
        let from = terms.lender.clone();
        terms.lender = to.clone();
        let store = e.storage().instance();
        store.set(&DataKey::Terms, &terms);
        store.remove(&DataKey::Pending);

        e.events().publish((symbol_short!("accepted"),), (from, to));
    }

    /// Settle the note to its current holder.
    ///
    /// Only the current holder (`lender` — the original payee, or the last
    /// accepted endorsee) may call this, and only while the note is `Active`.
    /// The `iset_signature` — ISET's proof that the off-chain legal
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

    /// Read the pending endorsee, if an endorsement offer is outstanding.
    pub fn pending(e: Env) -> Option<Address> {
        e.storage().instance().get(&DataKey::Pending)
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
