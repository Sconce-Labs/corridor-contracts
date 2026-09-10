#![no_std]
//! `corridor_registry` — the directory of payment corridors and their KYC
//! policy. Corridor operators register a [`CorridorPolicy`] and control its
//! parameters (tier, accepted issuers, verifier, the bulk-revocation floor,
//! pause).
//!
//! Option B: there is no relayer and no synced Midnight root. Eligibility is
//! proven against an issuer's Grumpkin Schnorr signature.

mod events;
use events::{AdminTransferred, MinCredEpochSet, PausedSet, PolicyUpdated, RegisteredEvent};

use corridor_types::{CorridorPolicy, Error};
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

#[contracttype]
enum DataKey {
    Admin,
    PendingAdmin,
    Policy(BytesN<32>),
}

fn read_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

#[contract]
pub struct CorridorRegistry;

#[contractimpl]
impl CorridorRegistry {
    /// Deploy-time constructor.
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn admin(env: Env) -> Address {
        read_admin(&env)
    }

    // ── admin transfer: propose → accept ───────────────────────────────────

    /// Step 1: the current admin nominates a successor.
    pub fn propose_admin(env: Env, new_admin: Address) {
        read_admin(&env).require_auth();
        env.storage()
            .instance()
            .set(&DataKey::PendingAdmin, &new_admin);
    }

    /// Step 2: the nominee accepts. Prevents locking the role to a typo'd address.
    pub fn accept_admin(env: Env) -> Result<(), Error> {
        let pending: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .ok_or(Error::NoPendingAdmin)?;
        pending.require_auth();
        let old = read_admin(&env);
        env.storage().instance().set(&DataKey::Admin, &pending);
        env.storage().instance().remove(&DataKey::PendingAdmin);
        AdminTransferred { old, new: pending }.publish(&env);
        Ok(())
    }

    // ── corridors ─────────────────────────────────────────────────────────

    /// Register a new corridor. The caller must authorize as `policy.operator`.
    pub fn register(
        env: Env,
        corridor_id: BytesN<32>,
        policy: CorridorPolicy,
    ) -> Result<(), Error> {
        policy.operator.require_auth();
        if env
            .storage()
            .persistent()
            .has(&DataKey::Policy(corridor_id.clone()))
        {
            return Err(Error::PolicyExists);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Policy(corridor_id.clone()), &policy);
        RegisteredEvent {
            corridor_id,
            operator: policy.operator,
            min_tier: policy.min_tier,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_policy(env: Env, corridor_id: BytesN<32>) -> Result<CorridorPolicy, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Policy(corridor_id))
            .ok_or(Error::PolicyNotFound)
    }

    /// Update the operator-controlled fields. Operator auth required. The
    /// operator address is preserved.
    pub fn update_policy(
        env: Env,
        corridor_id: BytesN<32>,
        new_policy: CorridorPolicy,
    ) -> Result<(), Error> {
        let existing: CorridorPolicy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(corridor_id.clone()))
            .ok_or(Error::PolicyNotFound)?;
        existing.operator.require_auth();

        let merged = CorridorPolicy {
            operator: existing.operator,
            ..new_policy
        };
        env.storage()
            .persistent()
            .set(&DataKey::Policy(corridor_id.clone()), &merged);
        PolicyUpdated {
            corridor_id,
            min_tier: merged.min_tier,
            required_disclosures: merged.required_disclosures,
            min_cred_epoch: merged.min_cred_epoch,
            verifier: merged.verifier,
            vk_hash: merged.vk_hash,
            paused: merged.paused,
        }
        .publish(&env);
        Ok(())
    }

    /// Raise the bulk-revocation floor for a corridor. Monotonic — a lower
    /// value is rejected. Operator auth required.
    pub fn set_min_cred_epoch(
        env: Env,
        corridor_id: BytesN<32>,
        min_cred_epoch: u64,
    ) -> Result<(), Error> {
        let mut policy: CorridorPolicy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(corridor_id.clone()))
            .ok_or(Error::PolicyNotFound)?;
        policy.operator.require_auth();
        if min_cred_epoch < policy.min_cred_epoch {
            return Err(Error::RootEpochRegression);
        }
        policy.min_cred_epoch = min_cred_epoch;
        env.storage()
            .persistent()
            .set(&DataKey::Policy(corridor_id.clone()), &policy);
        MinCredEpochSet {
            corridor_id,
            min_cred_epoch,
        }
        .publish(&env);
        Ok(())
    }

    pub fn set_paused(env: Env, corridor_id: BytesN<32>, paused: bool) -> Result<(), Error> {
        let mut policy: CorridorPolicy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(corridor_id.clone()))
            .ok_or(Error::PolicyNotFound)?;
        policy.operator.require_auth();
        policy.paused = paused;
        env.storage()
            .persistent()
            .set(&DataKey::Policy(corridor_id.clone()), &policy);
        PausedSet {
            corridor_id,
            paused,
        }
        .publish(&env);
        Ok(())
    }
}

#[cfg(test)]
mod test;
