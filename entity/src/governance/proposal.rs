multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use multiversx_sc::api::KECCAK256_RESULT_LEN;

use super::events;
use crate::config::{self, ProposalOptionId, ProposalRole, Timestamp, UserId, GAS_LIMIT_WITHDRAW, POLL_MAX_OPTIONS};
use crate::config::ProposalId;
use crate::permission::{self, PermissionName, RoleName};
use crate::errors::PROPOSAL_NOT_ACTIVE;
use crate::permission::PermissionDetails;
use crate::permission::{Policy, PolicyMethod, ROLE_BUILTIN_LEADER};
use crate::plug;
use core::convert::TryFrom;

static ACTION_HASH_FIELDS_SEPARATOR: &[u8] = b"|";

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub struct Proposal<M: ManagedTypeApi> {
    pub id: ProposalId,
    pub proposer: UserId,
    pub content_hash: ManagedBuffer<M>,
    pub actions_hash: ManagedBuffer<M>,
    pub starts_at: u64,
    pub ends_at: u64,
    pub executed: bool,
    pub roles: ManagedVec<M, RoleName<M>>,
}

impl<M: ManagedTypeApi> Proposal<M> {
    pub fn has_actions(&self) -> bool {
        !self.actions_hash.is_empty()
    }

    pub fn is_canceled(&self) -> bool {
        self.ends_at == 0
    }

    // cancel
    pub fn set_canceled(&mut self) {
        self.ends_at = 0;
    }
}

#[type_abi]
#[derive(TopEncode, TopDecode)]
// TODO: use all these fields instead of global values within `get_proposal_status` and co
pub struct ProposalDetails<M: ManagedTypeApi> {
    pub token: Option<TokenIdentifier<M>>,
    pub plug: Option<ManagedAddress<M>>,
    pub quorum: BigUint<M>,
    pub permissions: ManagedVec<M, PermissionName<M>>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone)]
pub struct Action<M: ManagedTypeApi> {
    pub destination: ManagedAddress<M>,
    pub endpoint: ManagedBuffer<M>,
    pub value: BigUint<M>,
    pub payments: ManagedVec<M, EsdtTokenPayment<M>>,
    pub arguments: ManagedVec<M, ManagedBuffer<M>>,
    pub gas_limit: u64,
}

#[type_abi]
#[derive(TopEncode, TopDecode, PartialEq, Debug)]
pub enum ProposalStatus {
    Pending,
    Active,
    Defeated,
    Succeeded,
    Executed,
    Canceled,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug, Clone)]
pub enum VoteType {
    For = 1,
    Against = 2,
}

type IsFirstTimeSignature = bool;

#[multiversx_sc::module]
pub trait ProposalModule: config::ConfigModule + permission::PermissionModule + events::GovEventsModule + plug::PlugModule {
    fn create_proposal(
        &self,
        proposer: UserId,
        trusted_host_id: ManagedBuffer,
        content_hash: ManagedBuffer,
        content_sig: ManagedBuffer,
        actions_hash: ManagedBuffer,
        option_id: u8,
        starts_at: Timestamp,
        vote_weight: BigUint,
        roles: ManagedVec<RoleName<Self::Api>>,
    ) -> Proposal<Self::Api> {
        let proposal_id = self.next_proposal_id().get();

        self.require_proposed_via_trusted_host(proposer, &trusted_host_id, &content_hash, content_sig, &actions_hash, &roles);
        require!(!self.known_trusted_host_proposal_ids().contains(&trusted_host_id), "proposal already registered");

        let (allowed, policies) = self.can_propose(proposer, &actions_hash, &roles);
        require!(allowed, "action not allowed for user");

        let has_weighted_policy = self.has_token_weighted_policy(&policies);

        if has_weighted_policy {
            require!(vote_weight >= self.min_propose_weight().get(), "insufficient vote weight");
        }

        if !actions_hash.is_empty() {
            require!(actions_hash.len() == KECCAK256_RESULT_LEN, "invalid actions hash");
        }

        let voting_period_minutes = policies
            .iter()
            .map(|p| p.voting_period_minutes)
            .max()
            .unwrap_or_else(|| self.default_voting_period_minutes().get());

        let current_time = self.blockchain().get_block_timestamp();

        let starts_at = if starts_at != 0 {
            starts_at
        } else {
            current_time
        };

        require!(starts_at > current_time, "start time must be in the future"); // TODO: test

        let ends_at = starts_at + voting_period_minutes as u64 * 60;

        require!(!roles.is_empty(), "roles must be provided"); // TODO: test

        // TODO: check all roles exist

        // TODO: check user has all roles

        let proposal = Proposal {
            id: proposal_id,
            proposer,
            content_hash,
            starts_at,
            ends_at,
            executed: false,
            actions_hash,
            roles,
        };

        let proposal_details = ProposalDetails {
            token: self.get_gov_token_option(),
            plug: self.get_plug_option(),
            quorum: self.default_quorum().get(),
            permissions: ManagedVec::new(), // TODO
        };

        for role in proposal.roles.iter() {
            if has_weighted_policy {
                self.vote_for_role(&proposal, &role, VoteType::For, &vote_weight);
            }

            self.proposal_signers(proposal.id, &role).insert(proposer);
        }

        self.proposals(proposal_id).set(&proposal);
        self.proposal_details(proposal_id).set(&proposal_details);
        self.next_proposal_id().set(proposal_id + 1);
        self.cast_poll_vote(proposal.id.clone(), option_id, vote_weight.clone());
        self.known_trusted_host_proposal_ids().insert(trusted_host_id);
        self.emit_propose_event(proposer, &proposal, vote_weight, option_id);

        proposal
    }

    // TODO: test
    fn cancel_proposal(&self, mut proposal: Proposal<Self::Api>) {
        let status = self.get_proposal_status(&proposal);

        require!(status == ProposalStatus::Active, PROPOSAL_NOT_ACTIVE);

        proposal.set_canceled();

        self.proposals(proposal.id).set(&proposal);

        self.emit_cancel_event(&proposal);
    }

    fn get_proposal_status(&self, proposal: &Proposal<Self::Api>) -> ProposalStatus {
        let current_time = self.blockchain().get_block_timestamp();

        if proposal.is_canceled() {
            return ProposalStatus::Canceled;
        }

        if proposal.executed {
            return ProposalStatus::Executed;
        }

        if proposal.starts_at < current_time{
            return ProposalStatus::Pending;
        }

        let (meets_policy_requirements, has_weighted_policy) = self.are_policies_fulfilled(&proposal);

        // When policies are fulfilled, and no time-based policies like
        // token-weighted or plug-based are applied that depend on the
        // voting period to end, the proposal is considered successful.
        if meets_policy_requirements && !has_weighted_policy {
            return ProposalStatus::Succeeded;
        }

        if current_time < proposal.ends_at {
            return ProposalStatus::Active;
        }

        if meets_policy_requirements {
            return ProposalStatus::Succeeded;
        }

        ProposalStatus::Defeated
    }

    fn are_policies_fulfilled(&self, proposal: &Proposal<Self::Api>) -> (bool, bool) {
        let details = self.proposal_details(proposal.id).get();
        let proposer_roles = self.user_roles(proposal.proposer);

        require!(!proposal.roles.is_empty(), "proposal has no defined roles");
        require!(!details.permissions.is_empty(), "proposal has no defined permissions");

        // Flags to check if all permissions are satisfied and if any token-weighted policies are applied.
        let mut are_fulfilled_all = true;
        let mut has_weighted_policy = false;

        // Evaluating each permission against the proposer's roles and associated policies.
        for permission in details.permissions.iter() {
            let is_fulfilled = proposer_roles
                .iter()
                .map(|role| {
                    if let Some(policy) = self.policies(&role).get(&permission) {
                        if policy.method == PolicyMethod::Weight {
                            has_weighted_policy = true;
                        }

                        match policy.method {
                            PolicyMethod::Weight => self.has_sufficient_votes(&proposal, &role, &policy.quorum),
                            PolicyMethod::One => self.proposal_signers(proposal.id, &role).contains(&proposal.proposer),
                            PolicyMethod::All => self.proposal_signers(proposal.id, &role).len() >= self.roles_member_amount(&role).get(),
                            PolicyMethod::Quorum => BigUint::from(self.proposal_signers(proposal.id, &role).len()) >= policy.quorum,
                            PolicyMethod::Majority => self.has_signer_majority_for_role(&proposal, &role),
                        }
                    } else {
                        true
                    }
                })
                .all(|fulfilled| fulfilled == true);

            if !is_fulfilled {
                are_fulfilled_all = false;
            }
        }

        (are_fulfilled_all, has_weighted_policy)
    }

    fn execute_actions(&self, actions: &ManagedVec<Action<Self::Api>>) {
        self.ensure_tokens_available_for_actions(&actions);

        for action in actions.iter() {
            let call = self.tx().to(action.destination).raw_call(action.endpoint).arguments_raw(action.arguments.into()).gas(action.gas_limit);

            if action.value > 0 {
                call.egld(action.value).transfer_execute();
            } else if !action.payments.is_empty() {
                call.multi_esdt(action.payments).transfer_execute();
            }
        }
    }

    fn ensure_tokens_available_for_actions(&self, actions: &ManagedVec<Action<Self::Api>>) {
        let mut token_map = ManagedMap::new();

        for action in actions.iter() {
            for payment in action.payments.iter() {
                let mut key = ManagedBuffer::new();
                key.append(payment.token_identifier.as_managed_buffer());
                key.append_bytes(&payment.token_nonce.to_be_bytes());

                if !token_map.contains(&key) {
                    let val = payment.amount.to_bytes_be_buffer();

                    // token_map.put(&key, val); // TODO
                }

                // let current = token_map.get(key.into());
                // let new = current + &payment.amount;
                // token_map.put(key, new);
            }
        }

        // Check if tokens are available for all accumulated payments
        for (key, amount) in token_map.iter() {
            self.require_tokens_available(&key.0, amount, key.1);
        }
    }

    fn vote(&self, voter: UserId, proposal: Proposal<Self::Api>, vote_type: VoteType, weight: BigUint, option_id: u8) {
        require!(weight > 0, "vote weight must be greater than 0");

        let min_vote_weight = self.min_vote_weight().get();

        // TODO: check if caller has role to vote

        require!(weight >= min_vote_weight, "not enought vote weight");
        require!(self.get_proposal_status(&proposal) == ProposalStatus::Active, PROPOSAL_NOT_ACTIVE);

        let intersecting_roles = self.get_user_intersecting_proposal_roles_or_fail(voter, &proposal);

        for role in intersecting_roles.iter() {
            self.vote_for_role(&proposal, &role, vote_type.clone(), &weight);
        }

        self.cast_poll_vote(proposal.id, option_id, weight.clone());
        self.emit_vote_event(voter, &proposal, vote_type, weight, option_id);
    }

    fn vote_for_role(&self, proposal: &Proposal<Self::Api>, role: &RoleName<Self::Api>, vote_type: VoteType, weight: &BigUint) {
        let mut role_info = if self.proposal_role_info(proposal.id, &role).is_empty() {
            ProposalRole::default()
        } else {
            self.proposal_role_info(proposal.id, &role).get()
        };

        match vote_type {
            VoteType::For => role_info.votes_for += weight,
            VoteType::Against => role_info.votes_against += weight,
        }

        self.proposal_role_info(proposal.id, &role).set(&role_info);
    }

    // TODO: test only signs for roles that are intersecting with the proposal
    // TODO: test fails if user does not have required roles
    fn sign(&self, proposal: ProposalId, option: ProposalOptionId) {
        let proposal = self.proposals(proposal).get();
        require!(self.get_proposal_status(&proposal) == ProposalStatus::Active, PROPOSAL_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        let signer = self.users().get_or_create_user(&caller);
        let intersecting_roles = self.get_user_intersecting_proposal_roles_or_fail(signer, &proposal);
        let mut is_first_time_sig = true;

        for role in intersecting_roles.iter() {
            let added = self.proposal_signers(proposal.id, &role).insert(signer);

            if !added {
                is_first_time_sig = false;
            }
        }

        // TODO: add test that checks can only sign once
        if is_first_time_sig {
            self.cast_poll_vote(proposal.id, option, BigUint::from(1u8));
            self.emit_sign_event(caller, &proposal, option);
        }
    }

    fn get_user_intersecting_proposal_roles_or_fail(&self, user: UserId, proposal: &Proposal<Self::Api>) -> ManagedVec<RoleName<Self::Api>> {
        let user_roles = self.user_roles(user);
        let intersecting_roles = proposal.roles.iter().filter(|role| user_roles.contains(role)).collect();

        require!(!intersecting_roles.is_empty(), "user does not have required roles");

        intersecting_roles
    }

    fn unsign_for_role(&self, signer: UserId, proposal: ProposalId, role: &ManagedBuffer) {
        self.proposal_signers(proposal, role).swap_remove(&signer);
    }

    fn cast_poll_vote(&self, proposal: ProposalId, option: ProposalOptionId, weight: BigUint) {
        if option == 0 || weight == 0 {
            return;
        }

        self.proposal_poll(proposal, option).update(|current| *current += weight);
    }

    fn withdraw_proposal_votes(&self, proposal: ProposalId) {
        let mut voter_ids_mapper = self.locked_voters(proposal);
        let safe_voter_ids = voter_ids_mapper.iter().collect::<ManagedVec<UserId>>();
        let mut withdrawn_amount = 0;
        let total_amount = safe_voter_ids.len();

        for voter in safe_voter_ids.iter() {
            if self.blockchain().get_gas_left() < GAS_LIMIT_WITHDRAW {
                break;
            }

            let withdrawn = self.withdraw_votes(voter, proposal);

            if withdrawn.is_ok() {
                withdrawn_amount += 1;
                voter_ids_mapper.swap_remove(&voter);
            }
        }

        self.emit_withdraw_progress_event(total_amount, withdrawn_amount);
    }

    fn withdraw_user_votes(&self, voter: UserId) {
        let mut proposal_ids_mapper = self.locked_votes_proposal_ids(voter);
        let safe_proposal_ids = proposal_ids_mapper.iter().collect::<ManagedVec<u64>>();
        let mut withdrawn_amount = 0;
        let total_amount = safe_proposal_ids.len();

        for proposal_id in safe_proposal_ids.iter() {
            if self.blockchain().get_gas_left() < GAS_LIMIT_WITHDRAW {
                break;
            }

            let withdrawn = self.withdraw_votes(voter, proposal_id);

            if withdrawn.is_ok() {
                withdrawn_amount += 1;
                proposal_ids_mapper.swap_remove(&proposal_id);
            }
        }

        self.emit_withdraw_progress_event(total_amount, withdrawn_amount);
    }

    fn withdraw_votes(&self, voter: UserId, proposal: ProposalId) -> Result<(), ()> {
        // TODO need to make sure that even persisted proposals can be withdrawn via test

        if self.proposals(proposal).is_empty() {
            return Ok(());
        }

        let proposal = self.proposals(proposal).get();
        let status = self.get_proposal_status(&proposal);

        if status == ProposalStatus::Active || status == ProposalStatus::Pending {
            return Err(());
        }

        let voter_address = self.users().get_user_address_unchecked(voter);
        let mut returnables: ManagedVec<EsdtTokenPayment> = ManagedVec::new();

        for locked_vote in self.locked_votes(proposal.id, voter).iter() {
            self.guarded_vote_tokens(&locked_vote.payment.token_identifier, locked_vote.payment.token_nonce)
                .update(|current| *current -= &locked_vote.payment.amount);

            returnables.push(locked_vote.payment);
        }

        self.locked_votes(proposal.id, voter).clear();

        self.emit_withdraw_event(&proposal);

        if !returnables.is_empty() {
            self.send().direct_multi(&voter_address, &returnables);
        }

        return Ok(());
    }

    fn persist_proposal_results(&self, proposal: ProposalId) {
        let proposal = self.get_proposal_or_fail(proposal);
        let status = self.get_proposal_status(&proposal);

        require!(status != ProposalStatus::Pending, "proposal is still pending");
        require!(status != ProposalStatus::Active, "proposal is still active");

        // cleanup
        self.proposals(proposal.id).clear();
        self.proposal_details(proposal.id).clear();
        self.proposal_nft_votes(proposal.id).clear();
        self.plug_votes(proposal.id).clear();

        for option in 1..=POLL_MAX_OPTIONS {
            self.proposal_poll(proposal.id, option).clear();
        }

        for role in proposal.roles.iter() {
            self.proposal_role_info(proposal.id, &role).clear();
            self.proposal_signers(proposal.id, &role).clear();
        }

        // persist
        self.proposal_results(proposal.id).set(status);
    }

    // TODO: tests
    /// A user should be able to propose when:
    /// - no actions are provided (no-op)
    /// - the proposer has any of the roles required by the policies
    /// - the DAO is leaderless: any user can propose
    fn can_propose(&self, proposer: UserId, actions_hash: &ManagedBuffer, policy_ids: &ManagedVec<RoleName<Self::Api>>) -> (bool, ManagedVec<Policy<Self::Api>>) {
        let has_actions = actions_hash.is_empty(); // no actions -> always allowed

        let policies = policy_ids.iter()
            .map(|policy_id| self.policies(policy_id).get())
            .collect::<ManagedVec<Policy<Self::Api>>>();

        let has_proposer_any_role = policies.iter()
            .any(|policy| self.user_roles(proposer).contains(&policy.role));

        let allowed = has_proposer_any_role || self.is_leaderless() || !has_actions;

        (allowed, policies)
    }

    fn calculate_actions_hash(&self, actions: &ManagedVec<Action<Self::Api>>) -> ManagedBuffer<Self::Api> {
        let mut serialized = ManagedBuffer::new();

        for action in actions.iter() {
            serialized.append(action.destination.as_managed_buffer());
            serialized.append_bytes(ACTION_HASH_FIELDS_SEPARATOR);
            serialized.append(&action.endpoint);
            serialized.append_bytes(ACTION_HASH_FIELDS_SEPARATOR);
            serialized.append(&action.value.to_bytes_be_buffer());
            serialized.append_bytes(ACTION_HASH_FIELDS_SEPARATOR);

            for payment in action.payments.iter() {
                serialized.append(payment.token_identifier.as_managed_buffer());
                serialized.append_bytes(&payment.token_nonce.to_be_bytes());
                serialized.append(&payment.amount.to_bytes_be_buffer());
            }

            serialized.append_bytes(ACTION_HASH_FIELDS_SEPARATOR);

            for arg in action.arguments.into_iter() {
                serialized.append(&arg);
            }
        }

        self.crypto().keccak256(&serialized).as_managed_buffer().clone()
    }

    // Note: used on execute and direct execute
    fn get_actions_execute_info(
        &self,
        user: UserId,
        actions: &ManagedVec<Action<Self::Api>>,
        has_member_approval: bool,
    ) -> (bool, ManagedVec<ManagedBuffer>) {
        let proposer_roles = self.user_roles(user);
        let mut applied_permissions = ManagedVec::new();

        for action in actions.iter() {
            let mut has_permission_for_action = false;

            for role in proposer_roles.iter() {
                for (permission, policy) in self.policies(&role).iter() {
                    let permission_details = self.permission_details(&permission).get();

                    if self.does_permission_apply_to_action(&permission_details, &action) {
                        applied_permissions.push(permission);
                        has_permission_for_action = has_member_approval || policy.method == PolicyMethod::One;
                    }
                }
            }

            // If after all checks, the action still does not have permission, return false.
            if !has_permission_for_action {
                return (false, applied_permissions);
            }
        }

        (true, applied_permissions)
    }

    fn does_permission_apply_to_action(&self, permission_details: &PermissionDetails<Self::Api>, action: &Action<Self::Api>) -> bool {
        // check value/EGLD mismatch
        if permission_details.value.is_some() && action.value > permission_details.value.unwrap() {
            return false;
        }

        // check destination mismatch
        if !permission_details.destination.is_zero() && action.destination != permission_details.destination {
            return false;
        }

        // check endpoint mismatch
        if !permission_details.endpoint.is_empty() && action.endpoint != permission_details.endpoint {
            return false;
        }

        // check arguments mismatch. ignored if permission contains no arguments.
        // the permission can scope the argument sequence down as far as needed:
        //      - passes: arg1, arg2 (permission) -> arg1, arg2, arg3 (action)
        //      - fails: arg1, arg2 (permission) -> arg1, arg3 (action)
        //      - fails: arg1, arg2 (permission) -> arg1 (action)
        if !permission_details.args.is_empty() {
            for (i, perm_arg) in permission_details.args.into_iter().enumerate() {
                if let Option::Some(arg_at_index) = action.arguments.try_get(i).as_deref() {
                    let applies = arg_at_index == &perm_arg;

                    if applies {
                        continue;
                    }
                }

                return false;
            }
        }

        // check payments mismatch. ignored if permission contains no payments.
        // returns false, if a payment is not in the permissions or exceeds payment amount.
        if !permission_details.payments.is_empty() {
            let applies = action.payments.into_iter().all(|payment| {
                if let Some(guard) = permission_details.payments.into_iter().find(|p| p.token_identifier == payment.token_identifier) {
                    payment.amount <= guard.amount
                } else {
                    false
                }
            });

            if !applies {
                return false;
            }
        }

        true
    }

    fn has_sufficient_votes(&self, proposal: &Proposal<Self::Api>, role: &RoleName<Self::Api>, quorum: &BigUint) -> bool {
        let proposal_role = self.proposal_role_info(proposal.id, &role).get();

        let total_votes = &proposal_role.votes_for + &proposal_role.votes_against;

        if total_votes == 0 {
            return false;
        }

        let vote_for_percent = &proposal_role.votes_for * &BigUint::from(100u64) / &total_votes;
        let vote_for_percent_to_pass = BigUint::from(50u64);

        vote_for_percent >= vote_for_percent_to_pass && &proposal_role.votes_for >= quorum
    }

    fn has_signer_majority_for_role(&self, proposal: &Proposal<Self::Api>, role: &ManagedBuffer) -> bool {
        let signer_count = self.proposal_signers(proposal.id, &role).len();
        let signer_majority = self.roles_member_amount(&role).get() / 2 + 1;

        signer_count > 0 && signer_count >= signer_majority
    }

    fn get_proposal_or_fail(&self, proposal: ProposalId) -> Proposal<Self::Api> {
        self.require_proposal_exists(proposal);

        self.proposals(proposal).get()
    }

    fn require_proposal_exists(&self, id: ProposalId) {
        require!(!self.proposals(id).is_empty(), "proposal does not exist");
    }

    fn require_proposed_via_trusted_host(
        &self,
        proposer: UserId,
        trusted_host_id: &ManagedBuffer,
        content_hash: &ManagedBuffer,
        content_sig: ManagedBuffer,
        actions_hash: &ManagedBuffer,
        roles: &ManagedVec<RoleName<Self::Api>>,
    ) {
        let entity = self.blockchain().get_sc_address();
        let proposer = self.users().get_user_address_unchecked(proposer);
        let trusted_host_sig = ManagedByteArray::try_from(content_sig).unwrap_or_default();

        let mut signable = ManagedBuffer::new();
        signable.append(proposer.as_managed_buffer());
        signable.append(entity.as_managed_buffer());
        signable.append(trusted_host_id);
        signable.append(content_hash);
        signable.append(actions_hash);

        for role in roles.into_iter() {
            signable.append(&role);
        }

        self.require_signed_by_trusted_host(&signable, &trusted_host_sig);
    }
}
