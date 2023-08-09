#![cfg(test)]
extern crate std;

use crate::{token, ErrorPropagationClient};

use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal};

fn create_token_contract<'a>(e: &Env, admin: &Address) -> token::Client<'a> {
    token::Client::new(e, &e.register_stellar_asset_contract(admin.clone()))
}

/// This is a testcase that shows what error is being propagated from wasm to the host
/// I have added those should_panics to show what error is being propagated
#[test]
#[should_panic(expected = "HostError: Error(Value, InvalidInput)")]
fn test_from_contract() {
    let e = Env::default();
    e.mock_all_auths();

    let from = Address::random(&e);
    let to = Address::random(&e);

    let token_a = create_token_contract(&e, &from);

    let error_propagation_client = ErrorPropagationClient::new(&e, &e.register_contract(None, crate::ErrorPropagation {}));
    error_propagation_client.transfers(
        &from,
        &to,
        &token_a.address,
        &1000i128
    );
}

// This is a replicated testcase from the token contract

#[test]
#[should_panic(expected = "HostError: Error(Value, InvalidInput)")]
fn transfer_insufficient_balance() {
    let e = Env::default();
    e.mock_all_auths();

    let from = Address::random(&e);
    let to = Address::random(&e);

    let token = create_token_contract(&e, &from);
    token.mint(&from, &1000);
    assert_eq!(token.balance(&from), 1000);
    token.transfer(&from, &to, &1001);
}

fn create_token<'a>(e: &Env, admin: &Address) -> token::Client<'a> {
    let token = token::Client::new(e, &e.register_stellar_asset_contract(admin.clone()));
    token.initialize(admin, &7, &"name".into_val(e), &"symbol".into_val(e));
    token
}

/// This test is the same as the one in the token contract
/// check out: ../token/src/test.rs:235
/// https://github.com/stellar/soroban-examples/blob/main/token/src/test.rs#L235

#[test]
#[should_panic(expected = "insufficient balance")]
fn transfer_insufficient_balance_same_test_as_in_token_contract() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::random(&e);
    let user1 = Address::random(&e);
    let user2 = Address::random(&e);
    let token = create_token(&e, &admin);

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.transfer(&user1, &user2, &1001);
}
