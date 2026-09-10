#![no_std]
//! `corridor_registry` — the directory of payment corridors and their KYC
//! policy. Corridor operators register a [`CorridorPolicy`]; a relayer keeps
//! the Midnight-derived roots fresh via [`post_root`].

mod events;
use events::{PausedSet, PolicyUpdated, Registered, RootPosted};

use corridor_types::{CorridorPolicy, Error};
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

#[contracttype]
enum DataKey {
    Admin,
    Policy(BytesN<32>),
}

fn zero32(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0u8; 32])
}

#[contract]
pub struct CorridorRegistry;

#[contractimpl]
impl CorridorRegistry {
    /// Deploy-time constructor. `admin` can later gate relayers (M5).
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Hand the admin role to a new address. Current admin authorizes.
    pub fn transfer_admin(env: Env, new_admin: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    /// Register a new corridor. The caller must authorize as `policy.operator`.
    /// Root fields on the incoming policy are ignored and start empty.
    pub fn register(
        env: Env,
        corridor_id: BytesN<32>,
        mut policy: CorridorPolicy,
    ) -> Result<(), Error> {
        policy.operator.require_auth();
        if env
            .storage()
            .persistent()
            .has(&DataKey::Policy(corridor_id.clone()))
        {
            return Err(Error::PolicyExists);
        }
        policy.credential_root = zero32(&env);
        policy.revocation_root = zero32(&env);
        policy.root_epoch = 0;
        env.storage()
            .persistent()
            .set(&DataKey::Policy(corridor_id.clone()), &policy);
        Registered {
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

    /// Update the operator-controlled fields. Operator auth required. Root
    /// fields and the operator address are preserved from the stored policy.
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
            credential_root: existing.credential_root,
            revocation_root: existing.revocation_root,
            root_epoch: existing.root_epoch,
            ..new_policy
        };
        env.storage()
            .persistent()
            .set(&DataKey::Policy(corridor_id.clone()), &merged);
        PolicyUpdated {
            corridor_id,
            min_tier: merged.min_tier,
            paused: merged.paused,
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

    /// Sync a fresh Midnight root onto a corridor's policy. `epoch` must strictly
    /// increase. MVP: any address may relay; every post emits a `RootPosted`
    /// event tagged with the relayer. M5 adds a relayer allowlist / quorum.
    pub fn post_root(
        env: Env,
        relayer: Address,
        corridor_id: BytesN<32>,
        credential_root: BytesN<32>,
        revocation_root: BytesN<32>,
        epoch: u64,
    ) -> Result<(), Error> {
        relayer.require_auth();
        let mut policy: CorridorPolicy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(corridor_id.clone()))
            .ok_or(Error::PolicyNotFound)?;
        if epoch <= policy.root_epoch {
            return Err(Error::RootEpochRegression);
        }
        policy.credential_root = credential_root.clone();
        policy.revocation_root = revocation_root.clone();
        policy.root_epoch = epoch;
        env.storage()
            .persistent()
            .set(&DataKey::Policy(corridor_id.clone()), &policy);
        RootPosted {
            corridor_id,
            relayer,
            epoch,
            credential_root,
            revocation_root,
        }
        .publish(&env);
        Ok(())
    }
}

#[cfg(test)]
mod test;
