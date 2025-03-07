multiversx_sc::imports!();

use crate::{config::{self}, permission::{self, ROLE_BUILTIN_LEADER}, plug};

const DEFAULT_DECIMALS: usize = 18;

#[multiversx_sc::module]
pub trait TokenModule: config::ConfigModule + plug::PlugModule + permission::PermissionModule {
    /// Initially configures the governance token if non is set already.
    /// It automatically calculates other governance setting defaults like quorum and minimum weight to propose.
    /// Can only be called by caller with leader role.
    #[endpoint(initGovToken)]
    fn init_gov_token_endpoint(&self, token_id: TokenIdentifier, supply: BigUint, lock_vote_tokens: bool) {
        require!(self.gov_token().is_empty(), "gov token is already set");
        require!(!self.is_plugged(), "already plugged");
        self.require_caller_has_leader_role();

        self.configure_governance_token(token_id, supply, lock_vote_tokens);
    }

    /// Change the governance token.
    /// Automatically calculates other governance setting defaults like quorum and minimum weight to propose.
    /// Can only be called by the contract itself.
    #[endpoint(changeGovToken)]
    fn change_gov_token_endpoint(&self, token_id: TokenIdentifier, supply: BigUint, lock_vote_tokens: bool) {
        self.require_caller_self();
        self.configure_governance_token(token_id, supply, lock_vote_tokens);
    }

    /// Remove the governance token.
    /// Entity must not be leaderless.
    /// Can only be called by the contract itself.
    #[endpoint(removeGovToken)]
    fn remove_gov_token_endpoint(&self) {
        self.require_caller_self();
        require!(!self.is_leaderless(), "not allowed when leaderless");

        let removed_gov_token = self.gov_token().take();
        self.lock_vote_tokens(&removed_gov_token).clear();
    }

    /// Issue and configure a fresh governance ESDT owned by the smart contract.
    /// It automatically calculates other governance setting defaults like quorum and minimum weight to propose.
    /// The initially minted tokens (supply) will be send to the caller.
    /// Can only be called by caller with leader role.
    /// Payment: EGLD in amount required by the protocol.
    #[payable("EGLD")]
    #[endpoint(issueGovToken)]
    fn issue_gov_token_endpoint(&self, token_name: ManagedBuffer, token_ticker: ManagedBuffer, supply: BigUint) {
        require!(self.gov_token().is_empty(), "governance token already set");
        require!(!self.is_plugged(), "already plugged");

        let caller = self.blockchain().get_caller();
        let user_id = self.users().get_user_id(&caller);
        let is_leader = self.user_roles(user_id).contains(&ManagedBuffer::from(ROLE_BUILTIN_LEADER));

        require!(is_leader, "only allowed for leader");
        require!(supply > 0, "amount must be greater than zero");

        let properties = FungibleTokenProperties {
            num_decimals: DEFAULT_DECIMALS,
            can_burn: false,
            can_mint: false,
            can_freeze: true,
            can_wipe: true,
            can_pause: true,
            can_change_owner: false,
            can_upgrade: true,
            can_add_special_roles: true,
        };

        self.tx()
            .to(ESDTSystemSCAddress)
            .typed(ESDTSystemSCProxy)
            .issue_fungible(self.call_value().egld_value().clone_value(), &token_name, &token_ticker, &supply, properties)
            .callback(self.callbacks().gov_token_issue_callback(&caller))
            .async_call_and_exit();
    }

    /// Set local Mint & Burn roles of the governance token for the smart contract.
    /// Usually called after `issueGovToken`.
    #[endpoint(setGovTokenLocalRoles)]
    fn set_gov_token_local_roles_endpoint(&self) {
        require!(!self.gov_token().is_empty(), "gov token must be set");

        let gov_token = self.gov_token().get();
        let entity_address = self.blockchain().get_sc_address();
        let roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];

        self.tx()
            .to(ESDTSystemSCAddress)
            .typed(ESDTSystemSCProxy)
            .set_special_roles(&entity_address, &gov_token, (&roles[..]).into_iter().cloned())
            .async_call_and_exit();
    }

    /// Mint tokens of any ESDT locally.
    /// This call will fail if the smart contract does not have the `ESDTRoleLocalMint` for the provided token id.
    #[endpoint(mint)]
    fn mint_endpoint(&self, token: TokenIdentifier, nonce: u64, amount: BigUint) {
        self.require_caller_self();

        self.tx()
            .to(ToSelf)
            .typed(system_proxy::UserBuiltinProxy)
            .esdt_local_mint(&token, nonce, &amount)
            .async_call_and_exit();
    }

    /// Burn tokens of any ESDT locally.
    /// This call will fail if the smart contract does not have the `ESDTRoleLocalBurn` for the provided token id.
    #[endpoint(burn)]
    fn burn_endpoint(&self, token: TokenIdentifier, nonce: u64, amount: BigUint) {
        self.require_caller_self();
        self.require_tokens_available(&token, nonce, &amount); // TODO: TEST

        self.tx()
            .to(ToSelf)
            .typed(system_proxy::UserBuiltinProxy)
            .esdt_local_burn(&token, nonce, &amount)
            .async_call_and_exit();
    }

    #[payable("*")]
    #[callback]
    fn gov_token_issue_callback(&self, initial_caller: &ManagedAddress, #[call_result] result: ManagedAsyncCallResult<()>) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                let payment = self.call_value().single_esdt();
                self.configure_governance_token(payment.token_identifier, payment.amount, true);
            }
            ManagedAsyncCallResult::Err(_) => self.send_received_egld(&initial_caller),
        }
    }

    fn configure_governance_token(&self, gov_token_id: TokenIdentifier, supply: BigUint, lock_vote_tokens: bool) {
        self.try_change_governance_token(&gov_token_id);
        self.lock_vote_tokens(&gov_token_id).set(lock_vote_tokens);

        if supply == 0 {
            return;
        }

        let initial_quorum = if &supply > &BigUint::from(100u64) {
            &supply * &BigUint::from(5u64) / &BigUint::from(100u64) // 5% of supply
        } else {
            BigUint::from(1u64)
        };

        let initial_min_tokens_for_proposing = if &supply > &BigUint::from(100u64) {
            &supply / &BigUint::from(100u64) // 1% of supply
        } else {
            BigUint::from(1u64)
        };

        self.try_change_quorum(BigUint::from(initial_quorum));
        self.try_change_min_propose_weight(BigUint::from(initial_min_tokens_for_proposing));
    }

    fn send_received_egld(&self, to: &ManagedAddress) {
        let egld_received = self.call_value().egld_value().clone_value();
        if egld_received > 0 {
            self.tx().to(to).egld(egld_received).transfer();
        }
    }
}
