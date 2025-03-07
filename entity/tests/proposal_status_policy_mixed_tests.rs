use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use multiversx_sc::codec::multi_types::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_returns_defeated_when_one_of_two_policies_do_not_meet_quorum() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let sc_address = setup.contract.address_ref().clone();
    let proposer_address = setup.user_address.clone();
    let signer_one = setup.blockchain.create_user_account(&rust_biguint!(1));
    let signer_two = setup.blockchain.create_user_account(&rust_biguint!(1));
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));

            sc.create_permission(
                managed_buffer!(b"testperm1"),
                managed_biguint!(0),
                managed_address!(&sc_address),
                managed_buffer!(b"testendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_permission(
                managed_buffer!(b"testperm2"),
                managed_biguint!(0),
                managed_address!(&sc_address),
                managed_buffer!(b"testendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );

            sc.create_policy(
                managed_buffer!(b"testrole"),
                managed_buffer!(b"testperm1"),
                PolicyMethod::All,
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
            sc.assign_role(managed_address!(&signer_one), managed_buffer!(b"testrole"));
            sc.assign_role(managed_address!(&signer_two), managed_buffer!(b"testrole"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM + 1), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&sc_address),
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

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.sign_endpoint(proposal_id, OptionalValue::None);
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
