#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

#[derive(Clone, Copy)]
#[contracttype]
enum DataKey {
    Maker,
    Lender,
    Amount,
    Maturity,
    IsetPubkey,
}

#[contract]
struct EpnEscrow;

#[contractimpl]
impl EpnEscrow {
    // 1. Initialize the escrow
    pub fn init(
        e: Env,
        maker: Address,
        lender: Address,
        amount: i128,
        maturity: u64,
        iset_pubkey: BytesN<32>,
    ) {
        if amount <= 0 {
            panic!("Amount must be greater than 0");
        }
        
        maker.require_auth();

        e.storage().instance().set(&DataKey::Maker, &maker);
        e.storage().instance().set(&DataKey::Lender, &lender);
        e.storage().instance().set(&DataKey::Amount, &amount);
        e.storage().instance().set(&DataKey::Maturity, &maturity);
        e.storage().instance().set(&DataKey::IsetPubkey, &iset_pubkey);
    }

    // 2. Release funds (Mocked verification for hackathon speed)
    pub fn release_funds(e: Env, caller: Address, _iset_signature: BytesN<64>) {
        let lender: Address = e.storage().instance().get::<DataKey, Address>(&DataKey::Lender).expect("Lender not set");
        
        if caller != lender {
            panic!("Access denied: Only the lender can release funds");
        }
        caller.require_auth();

        e.storage().instance().set(&DataKey::Maker, &caller); 
    }

    // 3. Claim default
    pub fn claim_default(e: Env, caller: Address) {
        let maker: Address = e.storage().instance().get::<DataKey, Address>(&DataKey::Maker).expect("Maker not set");
        
        if caller != maker {
            panic!("Access denied: Only the maker can claim default");
        }
        caller.require_auth();

        let maturity: u64 = e.storage().instance().get::<DataKey, u64>(&DataKey::Maturity).expect("Maturity not set");
        let now = e.ledger().timestamp();
        
        if now <= maturity {
            panic!("Contract has not reached maturity yet");
        }

        e.storage().instance().set(&DataKey::Lender, &caller);
    }
}