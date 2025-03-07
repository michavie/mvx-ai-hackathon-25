use entity::config::*;
use entity::governance::*;
use multiversx_sc::codec::multi_types::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_votes_for_a_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_PROPOSE_WEIGHT),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    // vote for
    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);

            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(managed_biguint!(MIN_PROPOSE_WEIGHT + 25), proposal.votes_for);
            assert_eq!(managed_biguint!(0), proposal.votes_against);
            assert_eq!(
                managed_biguint!(MIN_PROPOSE_WEIGHT + 25),
                sc.guarded_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID), 0).get()
            );
            assert!(sc.withdrawable_proposal_ids(&managed_address!(&voter_address)).contains(&proposal.id));

            let withdrawable_mapper = sc.withdrawable_votes(proposal.id, &managed_address!(&voter_address)).get(1);
            assert_eq!(managed_token_id!(ENTITY_GOV_TOKEN_ID), withdrawable_mapper.token_identifier);
            assert_eq!(managed_biguint!(25), withdrawable_mapper.amount);
        })
        .assert_ok();

    // same vote again to assert it adds up
    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);

            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(managed_biguint!(MIN_PROPOSE_WEIGHT + 50), proposal.votes_for);
            assert_eq!(managed_biguint!(0), proposal.votes_against);
            assert_eq!(
                managed_biguint!(MIN_PROPOSE_WEIGHT + 50),
                sc.guarded_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID), 0).get()
            );

            let withdrawable_mapper = sc.withdrawable_votes(proposal.id, &managed_address!(&voter_address)).get(2);
            assert_eq!(managed_token_id!(ENTITY_GOV_TOKEN_ID), withdrawable_mapper.token_identifier);
            assert_eq!(managed_biguint!(25), withdrawable_mapper.amount);
        })
        .assert_ok();
}

#[test]
fn it_votes_for_a_proposal_with_poll() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_address = setup.user_address.clone();
    let mut proposal_id = 0;
    let poll_option_id = 2u8;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_PROPOSE_WEIGHT),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    poll_option_id,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::Some(poll_option_id));

            assert_eq!(managed_biguint!(MIN_PROPOSE_WEIGHT + 25), sc.proposal_poll(proposal_id, poll_option_id).get());
        })
        .assert_ok();
}

#[test]
fn it_votes_against_a_proposal() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voter_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_PROPOSE_WEIGHT),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_against_endpoint(proposal_id, OptionalValue::None);

            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(managed_biguint!(MIN_PROPOSE_WEIGHT), proposal.votes_for);
            assert_eq!(managed_biguint!(25), proposal.votes_against);
            assert_eq!(
                managed_biguint!(MIN_PROPOSE_WEIGHT + 25),
                sc.guarded_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID), 0).get()
            );
            assert!(sc.withdrawable_proposal_ids(&managed_address!(&voter_address)).contains(&proposal.id));

            let withdrawable_mapper = sc.withdrawable_votes(proposal.id, &managed_address!(&voter_address)).get(1);
            assert_eq!(managed_token_id!(ENTITY_GOV_TOKEN_ID), withdrawable_mapper.token_identifier);
            assert_eq!(managed_biguint!(25), withdrawable_mapper.amount);
        })
        .assert_ok();

    // same vote again to assert it adds up
    setup
        .blockchain
        .execute_esdt_transfer(&voter_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_against_endpoint(proposal_id, OptionalValue::None);

            let proposal = sc.proposals(proposal_id).get();

            assert_eq!(managed_biguint!(MIN_PROPOSE_WEIGHT), proposal.votes_for);
            assert_eq!(managed_biguint!(50), proposal.votes_against);
            assert_eq!(
                managed_biguint!(MIN_PROPOSE_WEIGHT + 50),
                sc.guarded_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID), 0).get()
            );

            let withdrawable_mapper = sc.withdrawable_votes(proposal.id, &managed_address!(&voter_address)).get(2);
            assert_eq!(managed_token_id!(ENTITY_GOV_TOKEN_ID), withdrawable_mapper.token_identifier);
            assert_eq!(managed_biguint!(25), withdrawable_mapper.amount);
        })
        .assert_ok();
}

#[test]
fn it_fails_when_proposal_voting_period_has_ended() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_PROPOSE_WEIGHT),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_against_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_user_error(PROPOSAL_NOT_ACTIVE);
}

#[test]
fn it_fails_when_less_than_configured_min_vote_weight() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(MIN_PROPOSE_WEIGHT),
            |sc| {
                proposal_id = sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    ManagedBuffer::new(),
                    POLL_DEFAULT_ID,
                    MultiValueManagedVec::new(),
                );

                sc.try_change_min_vote_weight(managed_biguint!(50));
            },
        )
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&setup.user_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_user_error("not enought vote weight");
}
