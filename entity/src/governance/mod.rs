multiversx_sc::imports!();

use crate::config::{
    self, LockedVote, ProposalId, ProposalOptionId, Timestamp, UserId, MIN_PROPOSAL_VOTE_WEIGHT_DEFAULT, POLL_MAX_OPTIONS, QUORUM_DEFAULT, TOKEN_MAX_DECIMALS, VOTING_PERIOD_MINUTES_DEFAULT
};
use crate::errors::ALREADY_VOTED_WITH_TOKEN;
use crate::permission::{RoleName, ROLE_BUILTIN_MEMBER};
use crate::{permission, plug};
use proposal::{Action, ProposalStatus, VoteType};

use self::proposal::Proposal;

pub mod events;
pub mod proposal;
pub mod token;

use plug::plug_proxy::ProxyTrait as _;

#[multiversx_sc::module]
pub trait GovernanceModule:
    config::ConfigModule + plug::PlugModule + permission::PermissionModule + events::GovEventsModule + proposal::ProposalModule + token::TokenModule
{
    fn init_governance_module(&self) {
        self.next_proposal_id().set_if_empty(1);

        // Note: check to set later
        self.default_voting_period_minutes().set_if_empty(VOTING_PERIOD_MINUTES_DEFAULT);
        self.min_propose_weight().set_if_empty(BigUint::from(MIN_PROPOSAL_VOTE_WEIGHT_DEFAULT));
        self.default_quorum().set_if_empty(BigUint::from(QUORUM_DEFAULT));
    }

    /// Change the governance default quorum.
    /// Can only be called by the contract itself.
    #[endpoint(changeQuorum)]
    fn change_quorum_endpoint(&self, value: BigUint) {
        self.require_caller_self();
        self.try_change_default_quorum(value);
    }

    /// Change the minimum weight required to vote.
    /// Can only be called by the contract itself.
    #[endpoint(changeMinVoteWeight)]
    fn change_min_vote_weight_endpoint(&self, value: BigUint) {
        self.require_caller_self();
        self.try_change_min_vote_weight(value);
    }

    /// Change the minimumm weight required to create a proposal.
    /// Can only be called by the contract itself.
    #[endpoint(changeMinProposeWeight)]
    fn change_min_propose_weight_endpoint(&self, value: BigUint) {
        self.require_caller_self();
        self.try_change_min_propose_weight(value);
    }

    /// Change the default voting period.
    /// Can only be called by the contract itself.
    /// Arguments:
    ///     - value: voting period duration **in minutes**
    #[endpoint(changeVotingPeriodMinutes)]
    fn change_voting_period_in_minutes_endpoint(&self, value: usize) {
        self.require_caller_self();
        self.try_change_default_voting_period_minutes(value);
    }

    /// Set token nonces that are allowed to vote.
    /// Can only be called by the contract itself.
    #[endpoint(setRestrictedVoteNonces)]
    fn set_restricted_vote_nonces_endpoint(&self, nonces: MultiValueEncoded<u64>) {
        self.require_caller_self();
        self.restricted_vote_nonces().clear();
        self.restricted_vote_nonces().extend(nonces.into_iter());
    }

    /// Set the address of the plug smart contract.
    /// Can only be called by the contract itself.
    /// Can only be called once.
    #[endpoint(setPlug)]
    fn set_plug_endpoint(&self, address: ManagedAddress, quorum: BigUint, min_propose_weight: BigUint, weight_decimals: u8) {
        self.require_caller_self();
        require!(weight_decimals <= TOKEN_MAX_DECIMALS, "invalid weight decimals");

        self.plug_contract().set(&address);
        self.plug_weight_decimals().set(weight_decimals);
        self.try_change_default_quorum(quorum);
        self.try_change_min_propose_weight(min_propose_weight);
    }

    #[endpoint(eject)]
    fn eject_endpoint(&self, opt_trusted_host: OptionalValue<ManagedAddress>) {
        // TODO: check how to guard this

        if let OptionalValue::Some(trusted_host) = opt_trusted_host {
            self.trusted_host_address().set(&trusted_host);
        } else {
            self.trusted_host_address().clear();
        }
    }

    /// Create a proposal with optional actions
    /// Arguments:
    ///     - trusted_host_id: a unique id given by the trusted host
    ///     - content_hash: the hash of the proposed content to verify integrity on the frontend
    ///     - content_sig: signature provided by the trusted host
    ///     - actions_hash: the hash of serialized actions to verify on execution. leave empty if no actions attached
    ///     - option_id: unique id of poll option. 0 = None
    ///     - permissions (optional): a list of permissions (their unique names) to be verified on proposal execution
    /// Payment (optional):
    ///     - token id must be equal to configured governance token id
    ///     - amount must be greater than the min_propose_weight
    ///     - amount will be used to vote in favor (FOR) the proposal
    /// Returns an incremental proposal id
    #[payable("*")]
    #[endpoint(propose)]
    fn propose_endpoint(
        &self,
        trusted_host_id: ManagedBuffer,
        content_hash: ManagedBuffer,
        content_sig: ManagedBuffer,
        actions_hash: ManagedBuffer,
        option: ProposalOptionId,
        starts_at: Timestamp,
        roles: MultiValueEncoded<RoleName<Self::Api>>,
    ) -> u64 {
        let proposer_address = self.blockchain().get_caller();
        let proposer = self.users().get_or_create_user(&proposer_address);
        let payments = self.call_value().all_esdt_transfers().clone_value();

        self.persist_proposal_results();

        self.require_payments_with_gov_token(&payments);
        self.require_vote_tokens_allowed(&payments);

        let payment_weight = self.get_vote_weight_from_payments(&payments, proposer, Option::None);

        let proposal = self.create_proposal(
            proposer,
            trusted_host_id,
            content_hash,
            content_sig,
            actions_hash,
            option,
            starts_at,
            payment_weight.clone(),
            roles.to_vec(),
        );

        self.commit_vote_payments(proposer, &proposal, &payments);

        if self.is_plugged() {
            let token = self.get_gov_token_option();
            let plug = self.plug_contract().get();

            self.record_plug_vote(proposer, proposal.id);

            self.tx()
                .legacy_proxy_call(self.plug_proxy(plug).get_dao_vote_weight_view(&proposer_address, OptionalValue::from(token)))
                .callback(GovernanceModule::callbacks(self).propose_async_callback(
                    proposal.id,
                    proposer,
                    payment_weight,
                ))
                .async_call_and_exit();
        }

        proposal.id
    }

    /// Create a proposal via an asynchronous callback.
    /// Used majorly via the plugging feature.
    #[callback]
    fn propose_async_callback(
        &self,
        proposal: ProposalId,
        proposer: UserId,
        payment_weight: BigUint,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        let mut proposal = self.proposals(proposal.id).get();

        if result == ManagedAsyncCallResult::Err {
            self.cancel_proposal(proposal);
        }

        let plug_weight = result.unwrap_or_default();
        let total_weight = &payment_weight + &plug_weight;

        if total_weight >= self.min_vote_weight().get() {
            self.activate_proposal(proposal);
        } else {
            self.cancel_proposal(proposal);
        }

        // TODO: check to persist proposal results when implemented
    }

    /// Vote for of a proposal, optionally with a poll option.
    /// Payment (optional):
    ///     - token id must be equal to configured governance token id
    ///     - amount must be greater than the min_vote_weight
    ///     - ESDTs and SFTs will be deposited and locked until the voting period has ended
    ///     - NFTs will be recorded as a vote and immediately returned
    #[payable("*")]
    #[endpoint(voteFor)]
    fn vote_for_endpoint(&self, proposal: ProposalId, opt_option_id: OptionalValue<u8>) {
        let caller = self.blockchain().get_caller();
        let voter = self.users().get_or_create_user(&caller);
        let option_id = opt_option_id.into_option().unwrap_or_default();
        let payments = self.call_value().all_esdt_transfers();
        let proposal = self.proposals(proposal).get();
        let payment_weight = self.get_vote_weight_from_payments(&payments, voter, Option::Some(proposal.id));

        self.require_payments_with_gov_token(&payments);
        self.require_vote_tokens_allowed(&payments);
        self.commit_vote_payments(voter, &proposal, &payments);

        if self.is_plugged() {
            let token = self.get_gov_token_option();
            let plug = self.plug_contract().get();

            self.tx()
                .legacy_proxy_call(self.plug_proxy(plug).get_dao_vote_weight_view(&caller, OptionalValue::from(token)))
                .callback(GovernanceModule::callbacks(self).vote_async_callback(voter, payment_weight, proposal.id, VoteType::For, option_id))
                .async_call_and_exit();
        }

        self.vote(voter, proposal, VoteType::For, payment_weight, option_id);
    }

    /// Vote against a proposal.
    /// Payment (optional):
    ///     - token id must be equal to configured governance token id
    ///     - amount must be greater than the min_vote_weight
    ///     - ESDTs and SFTs will be deposited and locked until the voting period has ended
    ///     - NFTs will be recorded as a vote and immediately returned
    #[payable("*")]
    #[endpoint(voteAgainst)]
    fn vote_against_endpoint(&self, proposal: ProposalId, opt_option: OptionalValue<ProposalOptionId>) {
        let caller = self.blockchain().get_caller();
        let voter = self.users().get_or_create_user(&caller);
        let option_id = opt_option.into_option().unwrap_or_default();
        let payments = self.call_value().all_esdt_transfers();
        let proposal = self.proposals(proposal).get();
        let payment_weight = self.get_vote_weight_from_payments(&payments, voter, Option::Some(proposal.id));

        self.require_payments_with_gov_token(&payments);
        self.require_vote_tokens_allowed(&payments);
        self.commit_vote_payments(voter, &proposal, &payments);

        if self.is_plugged() {
            let token = self.get_gov_token_option();
            let plug = self.plug_contract().get();

            self.tx()
                .legacy_proxy_call(self.plug_proxy(plug).get_dao_vote_weight_view(&caller, OptionalValue::from(token)))
                .callback(GovernanceModule::callbacks(self).vote_async_callback(voter, payment_weight, proposal.id, VoteType::Against, option_id))
                .async_call_and_exit();
        }

        self.vote(voter, proposal, VoteType::Against, payment_weight, option_id);
    }

    /// Vote for or against a proposal via an asynchronous callback.
    /// The callback result must return the original caller's vote weight.
    /// Used majorly via the plugging feature.
    #[callback]
    fn vote_async_callback(
        &self,
        voter: UserId,
        payment_weight: BigUint,
        proposal: ProposalId,
        vote_type: VoteType,
        option_id: u8,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(vote_weight) => {
                let total_weight = if self.has_user_plug_voted(proposal, voter) {
                    payment_weight
                } else {
                    &payment_weight + &vote_weight
                };

                require!(total_weight > 0, "can not vote with 0 weight");
                let proposal = self.proposals(proposal).get();

                if self.is_plugged() {
                    self.record_plug_vote(voter, proposal.id);
                }

                self.vote(voter.clone(), proposal, vote_type, total_weight, option_id);
            }
            ManagedAsyncCallResult::Err(_) => {
                sc_panic!("failed to retrieve caller vote weight");
            }
        };
    }

    /// Sign a proposal, optionally with a poll option.
    /// This is often required by role members to approve actions protected by policies.
    #[endpoint(sign)]
    fn sign_endpoint(&self, proposal: ProposalId, opt_option_id: OptionalValue<u8>) {
        let option_id = opt_option_id.into_option().unwrap_or_default();
        self.sign(proposal, option_id);
    }

    /// Execute the actions of a succeeded proposal.
    /// This will update the proposals status to 'executed'.
    #[endpoint(execute)]
    fn execute_endpoint(&self, proposal: ProposalId, actions: MultiValueManagedVec<Action<Self::Api>>) {
        require!(!actions.is_empty(), "no actions to execute");
        require!(!self.proposals(proposal).is_empty(), "proposal not found");

        self.persist_proposal_results();

        let actions = actions.into_vec();
        let actions_hash = self.calculate_actions_hash(&actions);
        let mut proposal = self.proposals(proposal).get();
        require!(proposal.actions_hash == actions_hash, "actions have been corrupted");
        require!(!proposal.executed, "proposal has already been executed");

        let has_member_approval = self.get_proposal_status(&proposal) == ProposalStatus::Succeeded;
        let (allowed, permissions) = self.get_actions_execute_info(proposal.proposer, &actions, has_member_approval);
        require!(allowed, "no permission for action");
        require!(proposal.permissions == permissions, "untruthful permissions announced");

        proposal.executed = true;
        self.proposals(proposal.id).set(&proposal);

        self.execute_actions(&actions);
        self.emit_execute_event(&proposal);
    }

    /// Direct execute actions without a proposal.
    /// Requires the caller to have the required permissions.
    #[endpoint(directExecute)]
    fn direct_execute_endpoint(&self, actions: MultiValueManagedVec<Action<Self::Api>>) {
        require!(!actions.is_empty(), "no actions to execute");

        self.persist_proposal_results();

        let caller = self.blockchain().get_caller();
        let user = self.users().get_or_create_user(&caller);
        let actions = actions.into_vec();

        // proposal flow is skipped on direct executions,
        // so only unilaterally excutable actions are allowed.
        let has_member_approval = false;

        let (allowed, _) = self.get_actions_execute_info(user, &actions, has_member_approval);
        require!(allowed, "no permission for action");

        self.execute_actions(&actions);
        self.emit_direct_execute_event();
    }

    #[endpoint(cancelProposal)]
    fn cancel_proposal_endpoint(&self, proposal: ProposalId) {
        let caller = self.blockchain().get_caller();
        let caller = self.users().get_or_create_user(&caller);
        let proposal = self.proposals(proposal).get();
        require!(proposal.proposer == caller, "proposer must cancel proposal");

        self.cancel_proposal(proposal);

        self.persist_proposal_results();
    }

    /// Withdraw locked governance tokens once the proposals voting period has ended.
    /// Used by members who voted FOR or AGAINST a proposal using ESDTs.
    #[endpoint(withdraw)]
    fn withdraw_endpoint(&self) {
        let caller = self.blockchain().get_caller();
        let user = self.users().get_or_create_user(&caller);

        self.withdraw_user_votes(user);
    }

    /// Withdraw locked governance tokens once the proposals voting period has ended.
    /// Usable by anyone to withdraw tokens for all voters.
    #[endpoint(withdrawAll)]
    fn withdraw_all_endpoint(&self, proposal: ProposalId) {
        self.withdraw_proposal_votes(proposal);
    }

    #[view(getProposal)]
    fn get_proposal_view(&self, proposal: ProposalId) -> OptionalValue<MultiValue6<ManagedBuffer, ManagedBuffer, ManagedAddress, u64, u64, bool>> {
        if self.proposals(proposal).is_empty() {
            return OptionalValue::None;
        }

        let proposal = self.proposals(proposal).get();
        let proposer = self.users().get_user_address_unchecked(proposal.proposer);

        OptionalValue::Some(
            (
                proposal.content_hash,
                proposal.actions_hash,
                proposer,
                proposal.starts_at,
                proposal.ends_at,
                proposal.executed,
            )
                .into(),
        )
    }

    #[view(getProposalStatus)]
    fn get_proposal_status_view(&self, proposal: ProposalId) -> ProposalStatus {
        if !self.proposal_results(proposal).is_empty() {
            return self.proposal_results(proposal).get();
        }

        let proposal = self.get_proposal_or_fail(proposal);


        self.get_proposal_status(&proposal)
    }

    #[view(getProposalVotes)]
    fn get_proposal_votes_view(&self, proposal: ProposalId) -> MultiValue2<BigUint, BigUint> {
        self.require_proposal_exists(proposal);

        let member_role = ManagedBuffer::from(ROLE_BUILTIN_MEMBER);
        let role = self.proposal_role_info(proposal, &member_role).get();

        (role.votes_for, role.votes_against).into()
    }

    #[view(getProposalSigners)]
    fn get_proposal_signers_view(&self, proposal: ProposalId) -> MultiValueEncoded<ManagedAddress> {
        let proposal = self.proposals(proposal).get();
        let mut signers = MultiValueEncoded::new();

        // TODO: iterate over roles allowed to sign
        // for role in proposer_roles.iter() {
        //     for signer_id in self.proposal_signers(proposal.id, &role).iter() {
        //         let address = self.users().get_user_address_unchecked(signer_id);
        //         if !signers.to_vec().contains(&address) {
        //             signers.push(address);
        //         }
        //     }
        // }

        signers
    }

    #[view(getProposalSignatureRoleCounts)]
    fn get_proposal_signature_role_counts_view(&self, proposal: ProposalId) -> MultiValueEncoded<MultiValue2<ManagedBuffer, usize>> {
        let proposal = self.proposals(proposal).get();
        let mut signers = MultiValueEncoded::new();

        // TODO: iterate over roles allowed to sign
        // for role in proposer_roles.iter() {
        //     let signer_count = self.proposal_signers(proposal.id, &role).len();
        //     if signer_count > 0 {
        //         signers.push((role, signer_count).into());
        //     }
        // }

        signers
    }

    #[view(getProposalPollResults)]
    fn get_proposal_poll_results_view(&self, proposal: ProposalId) -> MultiValueEncoded<BigUint> {
        let mut results = MultiValueEncoded::new();

        for option_id in 1..=POLL_MAX_OPTIONS {
            results.push(self.proposal_poll(proposal, option_id).get());
        }

        results
    }

    fn get_vote_weight_from_payments(&self, payments: &ManagedVec<EsdtTokenPayment>, voter: UserId, opt_proposal: Option<ProposalId>,) -> BigUint {
        let mut total_weight = payments.into_iter().fold(BigUint::zero(), |carry, payment| carry + &payment.amount);

        if opt_proposal.is_none() {
            return total_weight;
        }

        let proposal = opt_proposal.unwrap();

        for locked_vote in self.locked_votes(proposal, voter).iter() {
            if !locked_vote.used.contains(&proposal) {
                total_weight += &locked_vote.payment.amount;
            }
        }

        total_weight
    }

    /// Processes received vote payment tokens.
    /// Either keeps track of them for withdrawals or sends them back immediately depending on the token type.
    /// - ESDTs will >always< be deposited/locked in the contract.
    /// - NFTs, SFTs & MetaESDTs are only locked if locked_vote_tokens is set to true (default).
    /// Fails if the NFT's nonce has been used to vote previously.
    fn commit_vote_payments(&self, user: UserId, proposal: &Proposal<Self::Api>, payments: &ManagedVec<EsdtTokenPayment>) {
        let mut returnables = ManagedVec::new();

        // TODO: TEST
        for (i, mut locked_vote) in self.locked_votes(proposal.id, user).iter().enumerate() {
            let new_unlocks_at = if locked_vote.unlocks_at < proposal.ends_at {
                proposal.ends_at
            } else {
                locked_vote.unlocks_at
            };

            locked_vote.unlocks_at = new_unlocks_at;
            locked_vote.used.push(proposal.id);
            self.locked_votes(proposal.id, user).set(i, &locked_vote);
        }

        for payment in payments.into_iter() {
            if payment.token_nonce == 0 || self.lock_vote_tokens(&payment.token_identifier).get() {
                self.guarded_vote_tokens(&payment.token_identifier, payment.token_nonce)
                    .update(|current| *current += &payment.amount);

                self.locked_voters(proposal.id).insert(user);
                self.locked_votes_proposal_ids(user).insert(proposal.id);
                self.locked_votes(proposal.id, user).push(&LockedVote {
                    unlocks_at: proposal.ends_at,
                    used: ManagedVec::from_single_item(proposal.id),
                    payment,
                });
            } else {
                let inserted = self.proposal_nft_votes(proposal.id).insert(payment.token_nonce);
                require!(inserted, ALREADY_VOTED_WITH_TOKEN);
                returnables.push(payment);
            }
        }

        if !returnables.is_empty() {
            let user = self.users().get_user_address_unchecked(user);

            self.tx().to(&user).multi_esdt(returnables).transfer();
        }
    }
}
