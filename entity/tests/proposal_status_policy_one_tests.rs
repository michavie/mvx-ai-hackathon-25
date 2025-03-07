use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_returns_succeeded_when_just_created_but_only_required_proposers_signature() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let sc_address = setup.contract.address_ref();
    let proposer_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));
            sc.create_permission(
                managed_buffer!(b"testperm"),
                managed_biguint!(0),
                managed_address!(sc_address),
                managed_buffer!(b"testendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"testrole"),
                managed_buffer!(b"testperm"),
                PolicyMethod::One,
                managed_biguint!(0),
                VOTING_PERIOD_MINUTES_DEFAULT,
            );
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"testrole"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(sc_address),
                endpoint: managed_buffer!(b"testendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"testperm")]);

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(ProposalStatus::Succeeded, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();
}

#[test]
fn it_succeeds_when_one_of_one_permission_policies_reaches_signer_quorum() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let sc_address = setup.contract.address_ref();
    let proposer_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));
            sc.create_permission(
                managed_buffer!(b"testperm"),
                managed_biguint!(0),
                managed_address!(sc_address),
                managed_buffer!(b"testendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"testrole"),
                managed_buffer!(b"testperm"),
                PolicyMethod::One,
                managed_biguint!(0),
                VOTING_PERIOD_MINUTES_DEFAULT,
            );
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"testrole"));
        })
        .assert_ok();

    // proposing also signs proposal
    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(sc_address),
                endpoint: managed_buffer!(b"testendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"testperm")]);

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(ProposalStatus::Succeeded, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();
}

#[test]
fn it_returns_defeated_when_one_of_two_permission_policies_does_not_meet_quorum_after_voting_period_ended() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let sc_address = setup.contract.address_ref();
    let proposer_address = setup.user_address.clone();
    let mut proposal_id = 0;

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));

            sc.create_permission(
                managed_buffer!(b"testperm1"),
                managed_biguint!(0),
                managed_address!(sc_address),
                managed_buffer!(b"testendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_permission(
                managed_buffer!(b"testperm2"),
                managed_biguint!(0),
                managed_address!(sc_address),
                managed_buffer!(b"testendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );

            sc.create_policy(
                managed_buffer!(b"testrole"),
                managed_buffer!(b"testperm1"),
                PolicyMethod::One,
                managed_biguint!(0),
                VOTING_PERIOD_MINUTES_DEFAULT,
            );
            sc.create_policy(
                managed_buffer!(b"testrole"),
                managed_buffer!(b"testperm2"),
                PolicyMethod::Quorum,
                managed_biguint!(5),
                VOTING_PERIOD_MINUTES_DEFAULT,
            );

            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"testrole"));
        })
        .assert_ok();

    // proposing also signs proposal
    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(MIN_PROPOSE_WEIGHT + 1), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(sc_address),
                endpoint: managed_buffer!(b"testendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"testperm1"), managed_buffer!(b"testperm2")]);

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
        })
        .assert_ok();
}
