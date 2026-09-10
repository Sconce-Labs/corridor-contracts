#![no_std]
//! UltraHonk proof verifier for Corridor — **skeleton (M3)**.
//!
//! Implements the same `verify(vk_hash, proof, public_inputs) -> bool` shape as
//! `verifier_mock`, so a corridor's policy can point at this address instead
//! with no change to `corridor_attestation`.
//!
//! The real verification is `bb`'s UltraHonk verifier ported to Soroban
//! (reference: `indextree/ultrahonk_soroban_contract`), using the Protocol 25
//! BN254 pairing + Poseidon2 host functions. This crate currently:
//!   * stores the verification key at deploy time
//!   * checks `vk_hash` matches the stored VK
//!   * returns `false` (nothing verifies yet) — tracked in
//!     https://github.com/Sconce-Labs/corridor-contracts/issues/1

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

    /// M3: run the UltraHonk verifier. For now this only enforces the VK pin
    /// and returns `false`.
    pub fn verify(
        env: Env,
        vk_hash: BytesN<32>,
        _proof: Bytes,
        _public_inputs: Vec<BytesN<32>>,
    ) -> bool {
        let stored: BytesN<32> = env.storage().instance().get(&DataKey::VkHash).unwrap();
        if vk_hash != stored {
            return false;
        }
        // TODO(M3): transcript, sumcheck, PCS opening, pairing check.
        false
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::{vec, Bytes, BytesN, Env};

    #[test]
    fn pins_the_vk_and_rejects_a_mismatched_hash() {
        let env = Env::default();
        let vk = Bytes::from_array(&env, &[1u8; 64]);
        let id = env.register(UltrahonkVerifier, (vk.clone(),));
        let client = UltrahonkVerifierClient::new(&env, &id);

        let good = client.vk_hash();
        let bad = BytesN::from_array(&env, &[0u8; 32]);
        let proof = Bytes::from_array(&env, &[0u8; 8]);
        let pi = vec![&env, BytesN::from_array(&env, &[0u8; 32])];

        // wrong vk_hash → false regardless
        assert!(!client.verify(&bad, &proof, &pi));
        // right vk_hash → still false until M3 lands the real verifier
        assert!(!client.verify(&good, &proof, &pi));
    }
}
