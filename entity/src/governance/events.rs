multiversx_sc::imports!();

use crate::config::{self, ProposalId, ProposalOptionId, UserId};

use super::proposal::{Proposal, VoteType};

#[multiversx_sc::module]
pub trait GovEventsModule: config::ConfigModule {
    fn emit_propose_event(&self, proposer: UserId, proposal: &Proposal<Self::Api>, weight: BigUint, poll_option: ProposalOptionId) {
        let proposer = self.users().get_user_address_unchecked(proposer);

        self.propose_event(proposer, proposal.id, weight, poll_option);
    }

    fn emit_vote_event(&self, voter: UserId, proposal: &Proposal<Self::Api>, vote_type: VoteType, weight: BigUint, poll_option: ProposalOptionId) {
        let voter = self.users().get_user_address_unchecked(voter);

        match vote_type {
            VoteType::For => {
                self.vote_for_event(voter, proposal.id, weight, poll_option);
            }
            VoteType::Against => {
                self.vote_against_event(voter, proposal.id, weight, poll_option);
            }
        }
    }

    fn emit_sign_event(&self, signer: ManagedAddress, proposal: &Proposal<Self::Api>, poll_option: ProposalOptionId) {
        self.sign_event(signer, proposal.id, poll_option);
    }

    fn emit_execute_event(&self, proposal: &Proposal<Self::Api>) {
        self.execute_event(self.blockchain().get_caller(), proposal.id);
    }

    fn emit_direct_execute_event(&self) {
        self.direct_execute_event(self.blockchain().get_caller());
    }

    fn emit_cancel_event(&self, proposal: &Proposal<Self::Api>) {
        self.cancel_event(self.blockchain().get_caller(), proposal.id);
    }

    fn emit_withdraw_event(&self, proposal: &Proposal<Self::Api>) {
        self.withdraw_event(self.blockchain().get_caller(), proposal.id);
    }

    fn emit_withdraw_progress_event(&self, total: usize, withdrawn: usize) {
        self.withdraw_progress_event(total, withdrawn);
    }

    #[event("propose")]
    fn propose_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: ProposalId, #[indexed] weight: BigUint, #[indexed] poll_option: u8);

    #[event("vote_for")]
    fn vote_for_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: ProposalId, #[indexed] weight: BigUint, #[indexed] poll_option: u8);

    #[event("vote_against")]
    fn vote_against_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: ProposalId, #[indexed] weight: BigUint, #[indexed] poll_option: u8);

    #[event("sign")]
    fn sign_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: ProposalId, #[indexed] poll_option: u8);

    #[event("execute")]
    fn execute_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: ProposalId);

    #[event("direct_execute")]
    fn direct_execute_event(&self, #[indexed] caller: ManagedAddress);

    #[event("cancel")]
    fn cancel_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: ProposalId);

    #[event("withdraw")]
    fn withdraw_event(&self, #[indexed] caller: ManagedAddress, #[indexed] proposal: ProposalId);

    #[event("withdraw_progress")]
    fn withdraw_progress_event(&self, #[indexed] total: usize, #[indexed] withdrawn: usize);
}
