#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env,
};

struct Fixture {
    env: Env,
    client: EpnEscrowClient<'static>,
    maker: Address,
    lender: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(EpnEscrow, ());
    let client = EpnEscrowClient::new(&env, &id);
    let maker = Address::generate(&env);
    let lender = Address::generate(&env);
    Fixture {
        env,
        client,
        maker,
        lender,
    }
}

fn zeros32(e: &Env) -> BytesN<32> {
    BytesN::from_array(e, &[0u8; 32])
}
fn zeros64(e: &Env) -> BytesN<64> {
    BytesN::from_array(e, &[0u8; 64])
}

const AMOUNT: i128 = 550_000;
const MATURITY: u64 = 7_776_000; // +90 days from ledger epoch

#[test]
fn init_sets_active_and_terms() {
    let f = setup();
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &MATURITY, &zeros32(&f.env));

    assert_eq!(f.client.status(), Status::Active);
    let terms = f.client.terms();
    assert_eq!(terms.maker, f.maker);
    assert_eq!(terms.lender, f.lender);
    assert_eq!(terms.amount, AMOUNT);
    assert_eq!(terms.maturity, MATURITY);
}

#[test]
fn lender_settles_the_note() {
    let f = setup();
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &MATURITY, &zeros32(&f.env));
    f.client.release_funds(&f.lender, &zeros64(&f.env));

    assert_eq!(f.client.status(), Status::Released);
}

#[test]
fn only_the_lender_may_settle() {
    let f = setup();
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &MATURITY, &zeros32(&f.env));

    let err = f
        .client
        .try_release_funds(&f.maker, &zeros64(&f.env))
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::Unauthorized.into());
}

#[test]
fn cannot_settle_twice() {
    let f = setup();
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &MATURITY, &zeros32(&f.env));
    f.client.release_funds(&f.lender, &zeros64(&f.env));

    let err = f
        .client
        .try_release_funds(&f.lender, &zeros64(&f.env))
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::NotActive.into());
}

#[test]
fn zero_amount_is_rejected() {
    let f = setup();
    let err = f
        .client
        .try_init(&f.maker, &f.lender, &0, &MATURITY, &zeros32(&f.env))
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::InvalidAmount.into());
}

#[test]
fn default_before_maturity_is_rejected() {
    let f = setup();
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &1_000, &zeros32(&f.env));

    let err = f.client.try_claim_default(&f.maker).err().unwrap().unwrap();
    assert_eq!(err, Error::NotMatured.into());
}

#[test]
fn maker_claims_default_after_maturity() {
    let f = setup();
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &1_000, &zeros32(&f.env));
    f.env.ledger().with_mut(|l| l.timestamp = 2_000);

    f.client.claim_default(&f.maker);
    assert_eq!(f.client.status(), Status::Defaulted);
}

#[test]
fn only_the_maker_may_claim_default() {
    let f = setup();
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &1_000, &zeros32(&f.env));
    f.env.ledger().with_mut(|l| l.timestamp = 2_000);

    let err = f
        .client
        .try_claim_default(&f.lender)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::Unauthorized.into());
}

// ── endorsement: offer (endorse) + acceptance (accept) ──────────────────────

#[test]
fn holder_endorses_and_endorsee_accepts() {
    let f = setup();
    let bank = Address::generate(&f.env);
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &MATURITY, &zeros32(&f.env));

    f.client.endorse(&f.lender, &bank);
    assert_eq!(f.client.pending(), Some(bank.clone()));

    f.client.accept(&bank);
    assert_eq!(f.client.pending(), None);
    assert_eq!(f.client.terms().lender, bank);

    // the new holder settles
    f.client.release_funds(&bank, &zeros64(&f.env));
    assert_eq!(f.client.status(), Status::Released);
}

#[test]
fn old_holder_loses_control_after_acceptance() {
    let f = setup();
    let bank = Address::generate(&f.env);
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &MATURITY, &zeros32(&f.env));
    f.client.endorse(&f.lender, &bank);
    f.client.accept(&bank);

    // the previous holder can no longer settle…
    let err = f
        .client
        .try_release_funds(&f.lender, &zeros64(&f.env))
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::Unauthorized.into());

    // …and cannot endorse the note a second time (no double-dealing)
    let other = Address::generate(&f.env);
    let err = f
        .client
        .try_endorse(&f.lender, &other)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::Unauthorized.into());
}

#[test]
fn only_the_current_holder_may_endorse() {
    let f = setup();
    let bank = Address::generate(&f.env);
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &MATURITY, &zeros32(&f.env));

    let err = f
        .client
        .try_endorse(&f.maker, &bank)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::Unauthorized.into());
}

#[test]
fn only_the_named_endorsee_may_accept() {
    let f = setup();
    let bank = Address::generate(&f.env);
    let mallory = Address::generate(&f.env);
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &MATURITY, &zeros32(&f.env));
    f.client.endorse(&f.lender, &bank);

    let err = f.client.try_accept(&mallory).err().unwrap().unwrap();
    assert_eq!(err, Error::Unauthorized.into());
}

#[test]
fn accept_without_an_offer_is_rejected() {
    let f = setup();
    let bank = Address::generate(&f.env);
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &MATURITY, &zeros32(&f.env));

    let err = f.client.try_accept(&bank).err().unwrap().unwrap();
    assert_eq!(err, Error::NoPendingEndorsement.into());
}

#[test]
fn re_endorsing_replaces_the_outstanding_offer() {
    let f = setup();
    let bank_a = Address::generate(&f.env);
    let bank_b = Address::generate(&f.env);
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &MATURITY, &zeros32(&f.env));

    f.client.endorse(&f.lender, &bank_a);
    f.client.endorse(&f.lender, &bank_b); // revocable until accepted

    let err = f.client.try_accept(&bank_a).err().unwrap().unwrap();
    assert_eq!(err, Error::Unauthorized.into());

    f.client.accept(&bank_b);
    assert_eq!(f.client.terms().lender, bank_b);
}

#[test]
fn settled_note_cannot_be_endorsed() {
    let f = setup();
    let bank = Address::generate(&f.env);
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &MATURITY, &zeros32(&f.env));
    f.client.release_funds(&f.lender, &zeros64(&f.env));

    let err = f
        .client
        .try_endorse(&f.lender, &bank)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::NotActive.into());
}

#[test]
fn a_new_note_does_not_inherit_a_stale_offer() {
    let f = setup();
    let bank = Address::generate(&f.env);
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &MATURITY, &zeros32(&f.env));
    f.client.endorse(&f.lender, &bank);

    // a fresh note re-initializes the instance — the old offer must not survive
    f.client
        .init(&f.maker, &f.lender, &AMOUNT, &MATURITY, &zeros32(&f.env));
    assert_eq!(f.client.pending(), None);

    let err = f.client.try_accept(&bank).err().unwrap().unwrap();
    assert_eq!(err, Error::NoPendingEndorsement.into());
}
