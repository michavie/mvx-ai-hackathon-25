multiversx_sc::imports!();

use crate::config::{self, ProposalId, UserId};

#[multiversx_sc::module]
pub trait PlugModule: config::ConfigModule {
    #[view(hasUserPlugVoted)]
    fn has_user_plug_voted_view(&self, proposal: ProposalId, address: ManagedAddress) -> bool {
        let user = self.users().get_user_id(&address);

        self.has_user_plug_voted(proposal, user)
    }

    fn record_plug_vote(&self, voter: UserId, proposal: ProposalId) {
        self.plug_votes(proposal).insert(voter);
    }

    fn has_user_plug_voted(&self, proposal: ProposalId, user: UserId) -> bool {
        self.plug_votes(proposal).contains(&user)
    }

    #[view(getPlug)]
    fn get_plug_view(&self) -> MultiValue2<ManagedAddress, u8> {
        let plug_contract = self.plug_contract().get();
        let weight_decimals = self.plug_weight_decimals().get();

        (plug_contract, weight_decimals).into()
    }

    #[proxy]
    fn plug_proxy(&self, to: ManagedAddress) -> plug_proxy::Proxy<Self::Api>;
}

pub mod plug_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait EntityPlugContractProxy {
        #[view(getDaoVoteWeight)]
        fn get_dao_vote_weight_view(&self, address: ManagedAddress, token: OptionalValue<TokenIdentifier>) -> BigUint;
    }
}
