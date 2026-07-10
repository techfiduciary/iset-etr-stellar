#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env,
};

fn setup(e: &Env) -> (EpnEscrowClient<'_>, Address, Address) {
    let id = e.register(EpnEscrow, ());
    let client = EpnEscrowClient::new(e, &id);
    let maker = Address::generate(e);
    let lender = Address::generate(e);
    (client, maker, lender)
}

fn zeros32(e: &Env) -> BytesN<32> {
    BytesN::from_array(e, &[0u8; 32])
}
fn zeros64(e: &Env) -> BytesN<64> {
    BytesN::from_array(e, &[0u8; 64])
}

#[test]
fn init_then_lender_settles() {
    let e = Env::default();
    e.mock_all_auths();
    let (c, maker, lender) = setup(&e);
    c.init(&maker, &lender, &550_000, &7_776_000, &zeros32(&e));
    // Only the lender can release — this must succeed.
    c.release_funds(&lender, &zeros64(&e));
}

#[test]
#[should_panic(expected = "Only the lender")]
fn non_lender_cannot_settle() {
    let e = Env::default();
    e.mock_all_auths();
    let (c, maker, lender) = setup(&e);
    c.init(&maker, &lender, &550_000, &7_776_000, &zeros32(&e));
    c.release_funds(&maker, &zeros64(&e));
}

#[test]
#[should_panic(expected = "Amount must be greater than 0")]
fn zero_amount_rejected() {
    let e = Env::default();
    e.mock_all_auths();
    let (c, maker, lender) = setup(&e);
    c.init(&maker, &lender, &0, &7_776_000, &zeros32(&e));
}

#[test]
#[should_panic(expected = "has not reached maturity")]
fn default_before_maturity_rejected() {
    let e = Env::default();
    e.mock_all_auths();
    let (c, maker, lender) = setup(&e);
    // Ledger time starts at 0; maturity is in the future.
    c.init(&maker, &lender, &550_000, &1_000, &zeros32(&e));
    c.claim_default(&maker);
}

#[test]
fn default_after_maturity() {
    let e = Env::default();
    e.mock_all_auths();
    let (c, maker, lender) = setup(&e);
    c.init(&maker, &lender, &550_000, &1_000, &zeros32(&e));
    e.ledger().with_mut(|l| l.timestamp = 2_000);
    c.claim_default(&maker);
}

#[test]
#[should_panic(expected = "Only the maker")]
fn non_maker_cannot_claim_default() {
    let e = Env::default();
    e.mock_all_auths();
    let (c, maker, lender) = setup(&e);
    c.init(&maker, &lender, &550_000, &1_000, &zeros32(&e));
    e.ledger().with_mut(|l| l.timestamp = 2_000);
    c.claim_default(&lender);
}
