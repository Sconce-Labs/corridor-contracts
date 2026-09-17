#![no_std]
//! UltraHonk proof verifier for Corridor.
//!
//! Implements the `verify(vk_hash, proof, public_inputs) -> bool` interface
//! from `corridor_types::Verifier`, so a corridor's policy can point at this
//! address instead of `verifier_mock` with no change to `corridor_attestation`.
//!
//! Adapts `indextree/ultrahonk_soroban_contract` and the Barretenberg UltraHonk
//! verifier ported to Soroban, using the Protocol 25 BN254 pairing host functions.

extern crate alloc;

pub mod debug;
pub mod ec;
pub mod field;
pub mod hash;
pub mod relations;
pub mod shplemini;
pub mod sumcheck;
pub mod transcript;
pub mod types;
pub mod utils;
pub mod verifier;

pub const PROOF_FIELDS: usize = 456;
pub const PROOF_BYTES: usize = PROOF_FIELDS * 32;

pub use verifier::UltraHonkVerifier;

use soroban_sdk::{contract, contractimpl, contracttype, Bytes, BytesN, Env, Vec};

#[contracttype]
enum DataKey {
    Vk,
    VkHash,
}

#[contract]
pub struct UltrahonkVerifier;

#[contractimpl]
impl UltrahonkVerifier {
    /// Deploy with the serialized verification key. Its hash is what policies
    /// pin via `vk_hash`.
    pub fn __constructor(env: Env, vk: Bytes) {
        let hash = env.crypto().sha256(&vk).to_bytes();
        env.storage().instance().set(&DataKey::Vk, &vk);
        env.storage().instance().set(&DataKey::VkHash, &hash);
    }

    pub fn vk_hash(env: Env) -> BytesN<32> {
        env.storage().instance().get(&DataKey::VkHash).unwrap()
    }

    /// Run the UltraHonk verifier.
    pub fn verify(
        env: Env,
        vk_hash: BytesN<32>,
        proof: Bytes,
        public_inputs: Vec<BytesN<32>>,
    ) -> bool {
        let stored: BytesN<32> = match env.storage().instance().get(&DataKey::VkHash) {
            Some(h) => h,
            None => return false,
        };
        if vk_hash != stored {
            return false;
        }

        if proof.len() as usize != PROOF_BYTES {
            return false;
        }

        let vk_bytes: Bytes = match env.storage().instance().get(&DataKey::Vk) {
            Some(vk) => vk,
            None => return false,
        };

        let verifier = match UltraHonkVerifier::new(&env, &vk_bytes) {
            Ok(v) => v,
            Err(_) => return false,
        };

        let mut pi_bytes = Bytes::new(&env);
        for pi in public_inputs.iter() {
            pi_bytes.append(&pi.into());
        }

        match verifier.verify(&proof, &pi_bytes) {
            Ok(()) => true,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod test;
