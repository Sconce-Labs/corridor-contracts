#![no_std]
//! `corridor_attestation` — the heart of the Stellar side.
//!
//! `enter` takes an UltraHonk proof + its public inputs, binds them to the
//! corridor's on-chain policy, verifies the proof through the policy's verifier
//! contract, burns a per-corridor nullifier so a credential can't be reused on
//! the same corridor, and records a [`PassRecord`]. Corridor operators gate
//! payouts by calling `is_cleared`.

mod events;
use events::PassGranted;

use corridor_types::{
    CorridorPolicy, Error, PassRecord, PublicInputs, RegistryClient, VerifierClient,
};
use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, BytesN, Env, Vec};

#[contracttype]
enum DataKey {
    Registry,
    /// (corridor_id, nullifier) -> PassRecord
    Nullifier(BytesN<32>, BytesN<32>),
    /// corridor_id -> aggregate pass count
    Passes(BytesN<32>),
}

/// A Corridor pass is **one-time per credential per corridor, permanently** —
/// the `(corridor_id, nullifier)` entry must never disappear, or the same
/// credential could `enter()` again. Soroban persistent entries cannot be made
/// truly immortal (they are bounded by the network `max_entry_ttl`, ~1 year on
/// mainnet), so:
///   * every write bumps the TTL to this target (the SDK clamps to the network
///     max),
///   * `is_cleared` / `pass_record` reads also bump it, so an actively-monitored
///     pass stays alive indefinitely,
///   * operators needing multi-year assurance with no on-chain reads should
///     mirror `is_cleared` results into their own contract.
/// A future milestone moves spent nullifiers into a cheap-to-persist
/// accumulator (see corridor-contracts#5).
const PASS_TTL_LEDGERS: u32 = 6_312_000; // ~2 years at 10s ledgers; clamped to network max
const PASS_TTL_THRESHOLD: u32 = 3_000_000;

#[contract]
pub struct CorridorAttestation;

#[contractimpl]
impl CorridorAttestation {
    pub fn __constructor(env: Env, registry: Address) {
        env.storage().instance().set(&DataKey::Registry, &registry);
    }

    pub fn registry(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Registry).unwrap()
    }

    /// Present an eligibility proof for `corridor_id`.
    ///
    /// `public_inputs` is the raw field-element vector the verifier consumes;
    /// its layout is defined in `corridor_types` (`PI_*`).
    pub fn enter(
        env: Env,
        corridor_id: BytesN<32>,
        proof: Bytes,
        public_inputs: Vec<BytesN<32>>,
    ) -> Result<(), Error> {
        let registry_addr: Address = env.storage().instance().get(&DataKey::Registry).unwrap();
        let policy: CorridorPolicy =
            RegistryClient::new(&env, &registry_addr).get_policy(&corridor_id);

        if policy.paused {
            return Err(Error::PolicyPaused);
        }

        let pi = PublicInputs::decode(&env, &public_inputs)?;

        // 1. bind the proof to this corridor's current policy
        if pi.credential_root != policy.credential_root
            || pi.revocation_root != policy.revocation_root
        {
            return Err(Error::RootMismatch);
        }
        if pi.corridor_id != corridor_id {
            return Err(Error::CorridorMismatch);
        }
        if pi.min_tier != policy.min_tier {
            return Err(Error::MinTierMismatch);
        }
        if !vec_contains(&policy.accepted_issuers, &pi.issuer_id) {
            return Err(Error::IssuerNotAccepted);
        }
        let ts = env.ledger().timestamp();
        let skew = if ts > pi.now {
            ts - pi.now
        } else {
            pi.now - ts
        };
        if skew > policy.now_tolerance_secs {
            return Err(Error::StaleProofTime);
        }

        // 2. verify the ZK proof through the policy's verifier contract
        let ok = VerifierClient::new(&env, &policy.verifier).verify(
            &policy.vk_hash,
            &proof,
            &public_inputs,
        );
        if !ok {
            return Err(Error::ProofInvalid);
        }

        // 3. burn the nullifier (one pass per credential per corridor)
        let nk = DataKey::Nullifier(corridor_id.clone(), pi.nullifier.clone());
        if env.storage().persistent().has(&nk) {
            return Err(Error::NullifierUsed);
        }
        let record = PassRecord {
            tag: pi.disclosed_tag,
            ledger: env.ledger().sequence(),
            timestamp: ts,
            auditor_blob: pi.auditor_blob.clone(),
        };
        env.storage().persistent().set(&nk, &record);
        env.storage()
            .persistent()
            .extend_ttl(&nk, PASS_TTL_THRESHOLD, PASS_TTL_LEDGERS);

        // 4. bump the aggregate counter
        let pk = DataKey::Passes(corridor_id.clone());
        let passes: u64 = env.storage().persistent().get(&pk).unwrap_or(0);
        env.storage().persistent().set(&pk, &(passes + 1));
        env.storage()
            .persistent()
            .extend_ttl(&pk, PASS_TTL_THRESHOLD, PASS_TTL_LEDGERS);

        PassGranted {
            corridor_id,
            nullifier: pi.nullifier,
            tag: pi.disclosed_tag,
            passes: passes + 1,
        }
        .publish(&env);
        Ok(())
    }

    /// Cheap read for a corridor operator's payout contract: has this nullifier
    /// been granted a pass on this corridor? Bumps the entry's TTL so an
    /// actively-monitored pass never expires.
    pub fn is_cleared(env: Env, corridor_id: BytesN<32>, nullifier: BytesN<32>) -> bool {
        let nk = DataKey::Nullifier(corridor_id, nullifier);
        if env.storage().persistent().has(&nk) {
            env.storage()
                .persistent()
                .extend_ttl(&nk, PASS_TTL_THRESHOLD, PASS_TTL_LEDGERS);
            true
        } else {
            false
        }
    }

    pub fn pass_record(
        env: Env,
        corridor_id: BytesN<32>,
        nullifier: BytesN<32>,
    ) -> Option<PassRecord> {
        let nk = DataKey::Nullifier(corridor_id, nullifier);
        let rec: Option<PassRecord> = env.storage().persistent().get(&nk);
        if rec.is_some() {
            env.storage()
                .persistent()
                .extend_ttl(&nk, PASS_TTL_THRESHOLD, PASS_TTL_LEDGERS);
        }
        rec
    }

    pub fn passes(env: Env, corridor_id: BytesN<32>) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::Passes(corridor_id))
            .unwrap_or(0)
    }
}

fn vec_contains(v: &Vec<BytesN<32>>, needle: &BytesN<32>) -> bool {
    for item in v.iter() {
        if &item == needle {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod test;
