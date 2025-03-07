multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::{config::{self, UserId, VOTING_PERIOD_MINUTES_MAX}, plug};

pub const ROLE_BUILTIN_LEADER: &[u8] = b"leader";
pub const ROLE_BUILTIN_MEMBER: &[u8] = b"member";
pub const PERMISSION_WILDCARD: &[u8] = b"*";
pub const PERMISSION_NOOP: &[u8] = b"-";

pub type RoleName<M> = ManagedBuffer<M>;
pub type PermissionName<M> = ManagedBuffer<M>;
pub type PolicyId = u32;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct PermissionDetails<M: ManagedTypeApi> {
    pub value: Option<BigUint<M>>,
    pub destination: Option<ManagedAddress<M>>,
    pub endpoint: Option<ManagedBuffer<M>>,
    pub arguments: Option<ManagedVec<M, ManagedBuffer<M>>>,
    pub payments: Option<ManagedVec<M, EsdtTokenPayment<M>>>,
}

impl<M: ManagedTypeApi> PermissionDetails<M> {
    pub fn wildcard() -> Self {
        Self {
            value: None,
            destination: None,
            endpoint: None,
            arguments: None,
            payments: None,
        }
    }

    // No-op permission have all fields set to default values.
    pub fn noop() -> Self {
        Self {
            value: Some(BigUint::zero()),
            destination: Some(ManagedAddress::zero()),
            endpoint: Some(ManagedBuffer::new()),
            arguments: Some(ManagedVec::new()),
            payments: Some(ManagedVec::new()),
        }
    }

    pub fn is_noop(&self) -> bool {
        self.value.as_ref().map_or(true, |v| v == &0)
        && self.destination.as_ref().map_or(false, |d| d.is_zero())
        && self.endpoint.as_ref().map_or(false, |e| e.is_empty())
        && self.arguments.as_ref().map_or(false, |a| a.is_empty())
        && self.payments.as_ref().map_or(false, |p| p.is_empty())
    }

    pub fn is_wildcard(&self) -> bool {
        self.value.is_none()
        && self.destination.is_none()
        && self.endpoint.is_none()
        && self.arguments.is_none()
        && self.payments.is_none()
    }
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct Policy<M: ManagedTypeApi> {
    pub method: PolicyMethod,
    pub quorum: BigUint<M>,
    pub voting_period_minutes: usize,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Copy, PartialEq, Debug, ManagedVecItem)]
pub enum PolicyMethod {
    Weight,
    One,
    All,
    Quorum,
    Majority,
}

impl PolicyMethod {
    pub fn to_name(&self) -> &[u8] {
        match self {
            PolicyMethod::Weight => b"weight",
            PolicyMethod::One => b"one",
            PolicyMethod::All => b"all",
            PolicyMethod::Quorum => b"quorum",
            PolicyMethod::Majority => b"majority",
        }
    }
}

#[multiversx_sc::module]
pub trait PermissionModule: config::ConfigModule + plug::PlugModule {
    fn init_permission_module(&self, leader: ManagedAddress) {
        self.configure_leader_role(leader);
    }

    /// Assigns the creator of the entity, creates a wildcard permission, and connects the two with a majority policy.
    fn configure_leader_role(&self, leader: ManagedAddress) {
        let role = ManagedBuffer::from(ROLE_BUILTIN_LEADER);
        let permission = ManagedBuffer::from(PERMISSION_WILDCARD);

        self.assign_role(leader.clone(), role.clone());
        self.create_permission(permission.clone(), Option::None, Option::None, Option::None, Option::None, Option::None);
        self.create_policy(role, permission, PolicyMethod::Majority, BigUint::zero(), self.default_voting_period_minutes().get());
    }

    fn configure_member_role(&self) {
        let role = ManagedBuffer::from(ROLE_BUILTIN_MEMBER);
        let permission = ManagedBuffer::from(PERMISSION_NOOP);

        self.create_role(role.clone());
        self.create_permission(permission.clone(), Option::Some(BigUint::zero()), Option::Some(ManagedAddress::zero()), Option::Some(ManagedBuffer::new()), Option::Some(ManagedVec::new()), Option::Some(ManagedVec::new()));
        self.create_policy(role, permission, PolicyMethod::Weight, BigUint::zero(), self.default_voting_period_minutes().get());
    }

    #[endpoint(createRole)]
    fn create_role_endpoint(&self, role: ManagedBuffer) {
        self.require_caller_self();
        self.create_role(role);
    }

    /// Remove a custom role.
    /// Will also unassign all users belonging to that role.
    /// Can only be called by the contract itself.
    #[endpoint(removeRole)]
    fn remove_role_endpoint(&self, role: ManagedBuffer) {
        self.require_caller_self();
        self.remove_role(role);
    }

    /// Assign a custom role to the given user.
    /// Can only be called by the contract itself.
    #[endpoint(assignRole)]
    fn assign_role_endpoint(&self, role: ManagedBuffer, address: ManagedAddress) {
        self.require_caller_self();
        self.assign_role(address, role);
    }

    /// Unassign a custom role from the given user.
    /// Can only be called by the contract itself.
    #[endpoint(unassignRole)]
    fn unassign_role_endpoint(&self, role: ManagedBuffer, address: ManagedAddress) {
        self.require_caller_self();
        self.unassign_role(address, role);
    }

    /// Create a permission.
    /// This permission can later be connected to custom roles through a policy.
    /// Can only be called by the contract itself.
    #[endpoint(createPermission)]
    fn create_permission_endpoint(
        &self,
        permission: PermissionName<Self::Api>,
        value: Option<BigUint>,
        destination: Option<ManagedAddress>,
        endpoint: Option<ManagedBuffer>,
        args: Option<ManagedVec<ManagedBuffer>>,
        payments: Option<ManagedVec<EsdtTokenPayment>>,
    ) {
        self.require_caller_self();
        require!(permission != ManagedBuffer::from(PERMISSION_WILDCARD), "wildcard permission cannot be created");
        require!(permission != ManagedBuffer::from(PERMISSION_NOOP), "noop permission cannot be created");

        self.create_permission(permission, value, destination, endpoint, args, payments);
    }

    /// Remove a permission.
    /// Can only be called by the contract itself.
    #[endpoint(removePermission)]
    fn remove_permission_endpoint(&self, permission: PermissionName<Self::Api>) {
        self.require_caller_self();
        require!(self.permissions().contains(&permission), "permission does not exist");

        self.permissions().swap_remove(&permission);
        self.permission_details(&permission).clear();
    }

    /// Create a policy that requires role members to vote based on the provided parameters in order to invoke the permission.
    /// Can only be called by the contract itself.
    #[endpoint(createPolicyWeighted)]
    fn create_policy_weighted_endpoint(&self, role: RoleName<Self::Api>, permission: PermissionName<Self::Api>,  opt_quorum: Option<BigUint>, opt_voting_period_minutes: Option<usize>) {
        self.require_caller_self();
        self.require_weighted_gov_method(); // TODO: TEST

        let quorum = opt_quorum.unwrap_or_else(|| self.default_quorum().get());
        require!(quorum > 0, "quorum must be greater than zero"); // TODO: TEST

        let voting_period_minutes = opt_voting_period_minutes.unwrap_or_else(|| self.default_voting_period_minutes().get());
        require!(voting_period_minutes > 0, "voting period must be greater than zero"); // TODO: TEST
        require!(voting_period_minutes <= VOTING_PERIOD_MINUTES_MAX, "max voting period exceeded"); // TODO: TEST

        self.create_policy(role, permission, PolicyMethod::Weight, quorum, voting_period_minutes);
    }

    /// Create a policy that allows permissions to be invoked unilaterally.
    /// Can only be called by the contract itself.
    #[endpoint(createPolicyOne)]
    fn create_policy_one_endpoint(&self, role: RoleName<Self::Api>, permission: PermissionName<Self::Api>) {
        self.require_caller_self();
        self.create_policy(role, permission, PolicyMethod::One, BigUint::from(1u64), 0);
    }

    /// Create a policy that requires all role members to sign in order to invoke the permission.
    /// Can only be called by the contract itself.
    #[endpoint(createPolicyAll)]
    fn create_policy_all_endpoint(&self, role: RoleName<Self::Api>, permission: PermissionName<Self::Api>) {
        self.require_caller_self();
        self.create_policy(role, permission, PolicyMethod::All, BigUint::zero(), self.default_voting_period_minutes().get());
    }

    /// Create a policy that requires role members to reach a defined quorum in order to invoke the permission.
    /// Can only be called by the contract itself.
    #[endpoint(createPolicyQuorum)]
    fn create_policy_quorum_endpoint(&self, role: RoleName<Self::Api>, permission: PermissionName<Self::Api>, quorum: usize) {
        self.require_caller_self();
        self.create_policy(
            role,
            permission,
            PolicyMethod::Quorum,
            BigUint::from(quorum),
            self.default_voting_period_minutes().get(),
        );
    }

    /// Create a policy that requires role members to reach a majority in order to invoke the permission.
    /// Can only be called by the contract itself.
    #[endpoint(createPolicyMajority)]
    fn create_policy_majority_endpoint(&self, role: RoleName<Self::Api>, permission: PermissionName<Self::Api>) {
        self.require_caller_self();
        self.create_policy(
            role,
            permission,
            PolicyMethod::Majority,
            BigUint::zero(),
            self.default_voting_period_minutes().get(),
        );
    }

    #[endpoint(removePolicy)]
    fn remove_policy_endpoint(&self, role: RoleName<Self::Api>, permission: PermissionName<Self::Api>) {
        self.require_caller_self();
        require!(self.roles().contains(&role), "role does not exist");
        require!(self.permissions().contains(&permission), "permission does not exist");

        let policy = self.policies(&role).remove(&permission);
        require!(policy.is_some(), "policy does not exist");
    }

    #[view(getUserRoles)]
    fn get_user_roles_view(&self, address: ManagedAddress) -> MultiValueEncoded<RoleName<Self::Api>> {
        let user_id = self.users().get_user_id(&address);
        let mut roles = MultiValueEncoded::new();

        if user_id == 0 {
            return roles;
        }

        for role in self.user_roles(user_id).iter() {
            roles.push(role);
        }

        roles
    }

    #[view(getPermissions)]
    fn get_permissions_view(&self) -> MultiValueEncoded<PermissionDetails<Self::Api>> {
        let mut permissions = MultiValueEncoded::new();

        for permission in self.permissions().iter() {
            permissions.push(self.permission_details(&permission).get());
        }

        permissions
    }

    #[view(getPolicies)]
    fn get_policies_view(&self, role_name: ManagedBuffer) -> MultiValueEncoded<MultiValue4<ManagedBuffer, ManagedBuffer, BigUint, usize>> {
        let mut policies = MultiValueEncoded::new();

        for (permission_name, policy) in self.policies(&role_name).iter() {
            policies.push(
                (
                    permission_name,
                    ManagedBuffer::from(policy.method.to_name()),
                    policy.quorum,
                    policy.voting_period_minutes,
                )
                    .into(),
            );
        }

        policies
    }

    fn create_role(&self, role: RoleName<Self::Api>) {
        let created = self.roles().insert(role);

        require!(created, "role already exists")
    }

    fn remove_role(&self, role: RoleName<Self::Api>) {
        require!(self.roles().contains(&role), "role does not exist");

        self.roles().swap_remove(&role);
        self.roles_member_amount(&role).set(0);

        // TODO: remove the unassign responsibility from this function
        for user_id in 1..=self.users().get_user_count() {
            self.user_roles(user_id).swap_remove(&role);
        }
    }

    fn assign_role(&self, address: ManagedAddress, role: RoleName<Self::Api>) {
        if !self.roles().contains(&role) {
            self.create_role(role.clone());
        }

        let user_id = self.users().get_or_create_user(&address);

        if self.user_roles(user_id).insert(role.clone()) {
            self.roles_member_amount(&role).update(|current| *current += 1);
        }
    }

    fn unassign_role(&self, address: ManagedAddress, role: RoleName<Self::Api>) {
        require!(self.roles().contains(&role), "role does not exist");

        let user = self.users().get_or_create_user(&address);

        // TODO: enable and fix dependency
        // for proposal in self.active_proposals().iter() {
        //     self.unsign_for_role(user, proposal, &role); // TODO: TEST
        // }

        if self.user_roles(user).swap_remove(&role) {
            self.roles_member_amount(&role).update(|current| *current -= 1);
        }
    }

    fn create_permission(
        &self,
        permission: PermissionName<Self::Api>,
        value: Option<BigUint>,
        destination: Option<ManagedAddress>,
        endpoint: Option<ManagedBuffer>,
        arguments: Option<ManagedVec<ManagedBuffer>>,
        payments: Option<ManagedVec<EsdtTokenPayment>>,
    ) {
        let inserted = self.permissions().insert(permission.clone());
        require!(inserted, "permission already exists"); // TODO: insert

        self.permission_details(&permission).set(PermissionDetails {
            value,
            destination,
            endpoint,
            arguments,
            payments,
        });
    }

    fn create_policy(&self, role: RoleName<Self::Api>, permission: PermissionName<Self::Api>, method: PolicyMethod, quorum: BigUint, voting_period_minutes: usize) {
        require!(self.roles().contains(&role), "role does not exist");
        require!(self.permissions().contains(&permission), "permission does not exist");
        require!(!self.policies(&role).contains_key(&permission), "policy already exists"); // TODO: test

        self.policies(&role).insert(
            permission,
            Policy {
                method,
                quorum,
                voting_period_minutes,
            },
        );
    }

    fn has_role(&self, address: &ManagedAddress, role: &RoleName<Self::Api>) -> bool {
        let user_id = self.users().get_user_id(&address);

        if user_id == 0 {
            return false;
        }

        self.user_roles(user_id).contains(&role)
    }

    fn has_token_weighted_policy(&self, policies: &ManagedVec<Policy<Self::Api>>) -> bool {
        policies.iter().find(|p| p.method == PolicyMethod::Weight).is_some()
    }

    fn is_leaderless(&self) -> bool {
        let leader_role = ManagedBuffer::from(ROLE_BUILTIN_LEADER);
        let is_leaderless = self.roles_member_amount(&leader_role).get() == 0;

        is_leaderless
    }

    fn has_leader_role(&self, address: &ManagedAddress) -> bool {
        let leader_role = ManagedBuffer::from(ROLE_BUILTIN_LEADER);

        self.has_role(&address, &leader_role)
    }

    fn require_caller_has_leader_role(&self) {
        let caller = self.blockchain().get_caller();
        require!(self.has_leader_role(&caller), "caller must be leader");
    }

    #[view(getRoles)]
    #[storage_mapper("roles")]
    fn roles(&self) -> UnorderedSetMapper<RoleName<Self::Api>>;

    #[view(getRoleMemberAmount)]
    #[storage_mapper("roles_member_amount")]
    fn roles_member_amount(&self, role: &RoleName<Self::Api>) -> SingleValueMapper<usize>;

    #[storage_mapper("user_roles")]
    fn user_roles(&self, user: UserId) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[storage_mapper("permissions")]
    fn permissions(&self) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[storage_mapper("permission_details")]
    fn permission_details(&self, permission: &RoleName<Self::Api>) -> SingleValueMapper<PermissionDetails<Self::Api>>;

    #[storage_mapper("policies")]
    fn policies(&self, role: &RoleName<Self::Api>) -> MapMapper<ManagedBuffer<Self::Api>, Policy<Self::Api>>;
}
