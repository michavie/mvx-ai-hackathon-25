#![no_std]

multiversx_sc::imports!();

pub mod config;
pub mod governance;
pub mod permission;
pub mod plug;
pub mod errors;

use crate::permission::ROLE_BUILTIN_LEADER;

#[multiversx_sc::contract]
pub trait Entity:
    config::ConfigModule
    + permission::PermissionModule
    + plug::PlugModule
    + governance::GovernanceModule
    + governance::events::GovEventsModule
    + governance::proposal::ProposalModule
    + governance::token::TokenModule
{
    #[init]
    fn init(&self, trusted_host: ManagedAddress, leader: ManagedAddress) {
        self.trusted_host_address().set(&trusted_host);
        self.init_governance_module();
        self.init_permission_module(leader);
    }

    #[upgrade]
    fn upgrade(&self, trusted_host: ManagedAddress) {
        self.trusted_host_address().set(&trusted_host);
    }

    // TODO: add tests
    #[endpoint(setLeaderlessMode)]
    fn set_leaderless_mode_endpoint(&self) {
        self.require_caller_self();
        self.require_weighted_gov_method();

        // TODO: make sure all leaders are unassigned (without iterating over all users)

        let leader_role = ManagedBuffer::from(ROLE_BUILTIN_LEADER);

        self.remove_role(leader_role);
    }

    #[endpoint(changeVoteTokenLock)]
    fn change_vote_token_lock_endpoint(&self, token: TokenIdentifier, lock: bool) {
        self.require_caller_self();
        self.lock_vote_tokens(&token).set(lock);
    }

    #[view(getVersion)]
    fn version_view(&self) -> &'static [u8] {
        env!("CARGO_PKG_VERSION").as_bytes()
    }
}

mod dns_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait Dns {
        #[payable("EGLD")]
        #[endpoint]
        fn register(&self, name: &ManagedBuffer);
    }
}
