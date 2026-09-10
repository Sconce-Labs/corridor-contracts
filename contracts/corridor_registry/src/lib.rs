#![no_std]
//! `corridor_registry` — the directory of payment corridors and their KYC
//! policy. Corridor operators register a [`CorridorPolicy`]; an **allowlisted**
//! relayer keeps the Midnight-derived roots fresh via [`post_root`].

mod events;
use events::{AdminTransferred, PausedSet, PolicyUpdated, RegisteredEvent, RelayerSet, RootPosted};

use corridor_types::{CorridorPolicy, Error};
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

#[contracttype]
enum DataKey {
    Admin,
    PendingAdmin,
    /// relayer -> allowed? (absent = not allowed)
    Relayer(Address),
    Policy(BytesN<32>),
}

fn zero32(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0u8; 32])
}

fn read_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

#[contract]
pub struct CorridorRegistry;

#[contractimpl]
impl CorridorRegistry {
    /// Deploy-time constructor. `admin` manages the relayer allowlist.
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn admin(env: Env) -> Address {
        read_admin(&env)
    }

    // ── admin transfer: propose → accept ────────────────────────────────────

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

    // ── relayer allowlist ──────────────────────────────────────────────────

    /// Admin adds/removes a relayer permitted to call [`post_root`].
    pub fn set_relayer(env: Env, relayer: Address, allowed: bool) {
        read_admin(&env).require_auth();
        if allowed {
            env.storage()
                .persistent()
                .set(&DataKey::Relayer(relayer.clone()), &true);
        } else {
            env.storage()
                .persistent()
                .remove(&DataKey::Relayer(relayer.clone()));
        }
        RelayerSet { relayer, allowed }.publish(&env);
    }

    pub fn is_relayer(env: Env, relayer: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Relayer(relayer))
            .unwrap_or(false)
    }

    // ── corridors ─────────────────────────────────────────────────────────

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
    /// operator address and the synced root fields are preserved.
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
            required_disclosures: merged.required_disclosures,
            verifier: merged.verifier,
            vk_hash: merged.vk_hash,
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

    /// Sync a fresh Midnight root onto a corridor's policy. The `relayer` must be
    /// on the admin's allowlist and `epoch` must strictly increase. Every post
    /// emits a `RootPosted` event tagged with the relayer.
    pub fn post_root(
        env: Env,
        relayer: Address,
        corridor_id: BytesN<32>,
        credential_root: BytesN<32>,
        revocation_root: BytesN<32>,
        epoch: u64,
    ) -> Result<(), Error> {
        relayer.require_auth();
        if !env
            .storage()
            .persistent()
            .get(&DataKey::Relayer(relayer.clone()))
            .unwrap_or(false)
        {
            return Err(Error::RelayerNotAllowed);
        }
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
