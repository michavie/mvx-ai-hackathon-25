use multiversx_sc::api::ED25519_SIGNATURE_BYTE_LEN;

use crate::{governance::proposal::{Proposal, ProposalDetails, ProposalStatus}, permission::RoleName};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const VOTING_PERIOD_MINUTES_DEFAULT: usize = 4320; // 3 days
pub const VOTING_PERIOD_MINUTES_MAX: usize = 20_160; // 14 days
pub const MIN_PROPOSAL_VOTE_WEIGHT_DEFAULT: u64 = 1;
pub const QUORUM_DEFAULT: u64 = 1;

pub const POLL_MAX_OPTIONS: u8 = 20;

pub const GAS_LIMIT_SET_TOKEN_ROLES: u64 = 60_000_000;
pub const GAS_LIMIT_WITHDRAW: u64 = 1_000_000;

pub const TOKEN_MAX_DECIMALS: u8 = 18;

pub type UserId = usize;
pub type ProposalId = u64;
pub type ProposalOptionId = u8;
pub type Timestamp = u64;


#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct LockedVote<M: ManagedTypeApi> {
    pub unlocks_at: Timestamp,
    pub used: ManagedVec<M, ProposalId>,
    pub payment: EsdtTokenPayment<M>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct ProposalRole<M: ManagedTypeApi> {
    pub votes_for: BigUint<M>,
    pub votes_against: BigUint<M>,
}

impl<M: ManagedTypeApi> Default for ProposalRole<M> {
    fn default() -> Self {
        Self {
            votes_for: 0,
            votes_against: 0,
        }
    }
}

#[multiversx_sc::module]
pub trait ConfigModule {
    fn require_caller_self(&self) {
        let caller = self.blockchain().get_caller();
        let sc_address = self.blockchain().get_sc_address();
        require!(caller == sc_address, "action not allowed by user");
    }

    fn require_caller_trusted_host(&self) {
        let caller = self.blockchain().get_caller();
        let trusted_host_address = self.trusted_host_address().get();
        require!(caller == trusted_host_address, "action not allowed by user");
    }

    fn require_weighted_gov_method(&self) {
        let has_gov_token = !self.gov_token().is_empty();
        let has_plug = self.is_plugged();

        require!(has_gov_token || has_plug, "gov token or plug must be set");
    }

    fn require_payments_with_gov_token(&self, payments: &ManagedVec<EsdtTokenPayment<Self::Api>>) {
        let gov_token_id = self.gov_token().get();

        for payment in payments.into_iter() {
            require!(payment.token_identifier == gov_token_id, "invalid payment token");
        }
    }

    fn require_tokens_available(&self, token: &TokenIdentifier, nonce: u64, amount: &BigUint) {
        let protected = self.guarded_vote_tokens(token, nonce).get();
        let balance = self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token.clone()), nonce);
        let available = balance - protected;

        require!(amount <= &available, "not enough tokens available");
    }

    fn require_signed_by_trusted_host(&self, signable: &ManagedBuffer, signature: &ManagedByteArray<Self::Api, ED25519_SIGNATURE_BYTE_LEN>) {
        if self.trusted_host_address().is_empty() {
            return;
        }

        let trusted_host = self.trusted_host_address().get();
        let signable_hashed = self.crypto().keccak256(signable);

        self.crypto()
            .verify_ed25519(trusted_host.as_managed_buffer(), signable_hashed.as_managed_buffer(), &signature.as_managed_buffer());
    }

    fn require_vote_tokens_allowed(&self, payments: &ManagedVec<EsdtTokenPayment<Self::Api>>) {
        if self.restricted_vote_nonces().is_empty() {
            return;
        }

        for payment in payments.into_iter() {
            let allowed = self.restricted_vote_nonces().contains(&payment.token_nonce);
            require!(allowed, "vote token nonce is restricted");
        }
    }

    fn is_plugged(&self) -> bool {
        !self.plug_contract().is_empty()
    }

    fn get_gov_token_option(&self) -> Option<TokenIdentifier> {
        if self.gov_token().is_empty() {
            return None;
        }

        Some(self.gov_token().get())
    }

    fn get_plug_option(&self) -> Option<ManagedAddress> {
        if self.plug_contract().is_empty() {
            return None;
        }

        Some(self.plug_contract().get())
    }

    fn try_change_governance_token(&self, token_id: &TokenIdentifier) {
        require!(token_id.is_valid_esdt_identifier(), "invalid token id");
        self.gov_token().set(token_id);
    }

    fn try_change_default_quorum(&self, quorum: BigUint) {
        require!(quorum != 0, "invalid quorum");
        self.default_quorum().set(&quorum);
    }

    fn try_change_min_vote_weight(&self, vote_weight: BigUint) {
        require!(vote_weight != 0, "min vote weight can not be zero");
        self.min_vote_weight().set(&vote_weight);
    }

    fn try_change_min_propose_weight(&self, vote_weight: BigUint) {
        require!(vote_weight != 0, "min propose weight can not be zero");
        self.min_propose_weight().set(&vote_weight);
    }

    fn try_change_default_voting_period_minutes(&self, voting_period: usize) {
        require!(voting_period != 0, "voting period can not be zero");
        require!(voting_period <= VOTING_PERIOD_MINUTES_MAX, "max voting period exceeded");
        self.default_voting_period_minutes().set(&voting_period);
    }

    #[storage_mapper("users")]
    fn users(&self) -> UserMapper;

    #[view(getTrustedHostAddress)]
    #[storage_mapper("trusted_host_addr")]
    fn trusted_host_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getGovTokenId)]
    #[storage_mapper("gov_token_id")]
    fn gov_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getGuardedVoteTokens)]
    #[storage_mapper("guarded_vote_tokens")]
    fn guarded_vote_tokens(&self, token_id: &TokenIdentifier, nonce: u64) -> SingleValueMapper<BigUint>;

    #[view(isLockingVoteTokens)]
    #[storage_mapper("lock_vote_tokens")]
    fn lock_vote_tokens(&self, token_id: &TokenIdentifier) -> SingleValueMapper<bool>;

    #[storage_mapper("proposals")]
    fn proposals(&self, id: ProposalId) -> SingleValueMapper<Proposal<Self::Api>>;

    #[storage_mapper("proposal_details")]
    fn proposal_details(&self, id: ProposalId) -> SingleValueMapper<ProposalDetails<Self::Api>>;

    #[storage_mapper("proposal_role_info")]
    fn proposal_role_info(&self, id: ProposalId, role: &RoleName<Self::Api>) -> SingleValueMapper<ProposalRole<Self::Api>>;

    #[storage_mapper("proposal_results")]
    fn proposal_results(&self, proposal: ProposalId) -> SingleValueMapper<ProposalStatus>;

    #[view(getNextProposalId)]
    #[storage_mapper("proposals_id_counter")]
    fn next_proposal_id(&self) -> SingleValueMapper<ProposalId>;

    #[storage_mapper("proposal_signers")]
    fn proposal_signers(&self, proposal: ProposalId, role_name: &ManagedBuffer) -> UnorderedSetMapper<usize>;

    #[view(getProposalNftVotes)]
    #[storage_mapper("proposal_nft_votes")]
    fn proposal_nft_votes(&self, proposal: ProposalId) -> UnorderedSetMapper<u64>;

    #[storage_mapper("proposal_poll")]
    fn proposal_poll(&self, proposal: ProposalId, option: ProposalOptionId) -> SingleValueMapper<BigUint>;

    // DEPRECATED: not populated anymore, only used for withdrawals
    #[view(getWithdrawableVoters)]
    #[storage_mapper("withdrawable_voters")]
    fn withdrawable_voters(&self, proposal: ProposalId) -> UnorderedSetMapper<usize>;

    // DEPRECATED: not populated anymore, only used for withdrawals
    #[view(getWithdrawableProposalIds)]
    #[storage_mapper("withdrawable_proposal_ids")]
    fn withdrawable_proposal_ids(&self, voter: &ManagedAddress) -> UnorderedSetMapper<u64>;

    // DEPRECATED: not populated anymore, only used for withdrawals
    #[view(getWithdrawableVotes)]
    #[storage_mapper("withdrawable_votes")]
    fn withdrawable_votes(&self, proposal: ProposalId, voter: &ManagedAddress) -> VecMapper<EsdtTokenPayment>;

    #[view(getLockedVoters)]
    #[storage_mapper("locked_voters")]
    fn locked_voters(&self, proposal: ProposalId) -> UnorderedSetMapper<UserId>;

    #[view(getLockedVotesProposalIds)]
    #[storage_mapper("locked_votes_proposal_ids")]
    fn locked_votes_proposal_ids(&self, voter: UserId) -> UnorderedSetMapper<u64>;

    #[view(getLockedVotes)]
    #[storage_mapper("locked_votes")]
    fn locked_votes(&self, proposal: ProposalId, voter: UserId) -> VecMapper<LockedVote<Self::Api>>;

    #[storage_mapper("known_th_proposals_ids")]
    fn known_trusted_host_proposal_ids(&self) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[view(getQuorum)]
    #[storage_mapper("default_quorum")]
    fn default_quorum(&self) -> SingleValueMapper<BigUint>;

    #[view(getMinVoteWeight)]
    #[storage_mapper("min_vote_weight")]
    fn min_vote_weight(&self) -> SingleValueMapper<BigUint>;

    #[view(getMinProposeWeight)]
    #[storage_mapper("min_proposal_vote_weight")]
    fn min_propose_weight(&self) -> SingleValueMapper<BigUint>;

    #[view(getVotingPeriodMinutes)]
    #[storage_mapper("default_voting_period_minutes")]
    fn default_voting_period_minutes(&self) -> SingleValueMapper<usize>;

    #[view(getRestrictedVoteNonces)]
    #[storage_mapper("restricted_vote_nonces")]
    fn restricted_vote_nonces(&self) -> UnorderedSetMapper<u64>;

    #[storage_mapper("plug:contract")]
    fn plug_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("plug:weight_decimals")]
    fn plug_weight_decimals(&self) -> SingleValueMapper<u8>;

    #[storage_mapper("plug:votes")]
    fn plug_votes(&self, proposal: ProposalId) -> UnorderedSetMapper<UserId>;
}
