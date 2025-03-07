use entity::config::*;
use entity::governance::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_issues_a_governance_token() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0.5), |sc| {
            sc.issue_gov_token_endpoint(managed_buffer!(b"Token"), managed_buffer!(b"Token-123456"), managed_biguint!(100_000));

            assert!(!sc.quorum().is_empty());
            assert!(!sc.min_propose_weight().is_empty());
            assert_eq!(managed_token_id!(ENTITY_GOV_TOKEN_ID), sc.gov_token().get());
        })
        .assert_ok();
}

#[test]
fn it_fails_when_caller_not_a_leader() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let caller = setup.blockchain.create_user_account(&rust_biguint!(1));

    setup
        .blockchain
        .execute_tx(&caller, &setup.contract, &rust_biguint!(0.5), |sc| {
            sc.issue_gov_token_endpoint(managed_buffer!(b"Token"), managed_buffer!(b"Token-123456"), managed_biguint!(100_000));
        })
        .assert_user_error("only allowed for leader");
}

#[test]
fn it_fails_when_gov_token_has_already_been_issued() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0.5), |sc| {
            sc.gov_token().set(managed_token_id!(ENTITY_GOV_TOKEN_ID));

            sc.issue_gov_token_endpoint(managed_buffer!(b"Token"), managed_buffer!(b"Token-123456"), managed_biguint!(100_000));
        })
        .assert_user_error("governance token already set");
}
