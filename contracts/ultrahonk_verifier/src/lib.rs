#![no_std]
//! UltraHonk proof verifier for Corridor — **real verification (M3)**.
//!
//! Implements the same `verify(vk_hash, proof, public_inputs) -> bool` wire ABI
//! as `verifier_mock`, so a corridor's policy can point at this address with no
//! change to `corridor_attestation`. This crate deliberately does **not**
//! depend on `corridor_types`: the verification runs on soroban-sdk 28 (see
//! "SDK isolation" in `Cargo.toml`) while `corridor_types` is a workspace
//! SDK-25 crate, and the wire ABI is SDK-agnostic.
//!
//! # What is verified
//!
//! The full UltraHonk (UltraKeccakFlavor, non-ZK) decider, ported from
//! Barretenberg v0.87.0: proof deserialization with canonical-encoding checks,
//! Fiat–Shamir transcript, sumcheck, Shplemini (Gemini + Shplonk + KZG), and
//! the BN254 pairing check — all through the Soroban host's native `bn254`
//! crypto. The core lives in `crates/ultrahonk_core`, vendored byte-identical
//! from [`NethermindEth/ultrahonk-rust-verifier`] at
//! `097da17df8b0b971d71885566034958a8caebab4` (post OpenZeppelin-audit
//! remediation: 0 Critical/High/Medium, 5 Low — all remediated).
//!
//! # Trust model
//!
//! The VK is immutable and set once at deployment, validated by parsing before
//! storage. There is no admin key and no upgrade path. **The deployer is
//! solely responsible for supplying the correct VK** — anyone can deploy an
//! instance with an arbitrary VK, so callers must pin `vk_hash` (this
//! contract's `vk_hash()` of the known-good VK) in the corridor policy, which
//! `verify` enforces on every call.
//!
//! **Toolchain pin:** proof/VK bytes must come from Barretenberg **v0.87.0**
//! (`UltraKeccakFlavor`). A bb version mismatch is *not* caught by the
//! length checks — it surfaces as a failed transcript/pairing (i.e. proofs
//! simply do not verify). Prove the corridor circuit with bb 0.87.0 or
//! coordinate a core bump; see `crates/ultrahonk_core/VERIFIER_PROVENANCE.md`.
//!
//! # Failure semantics
//!
//! `verify` is **fail-closed**: any malformed VK reference, wrong `vk_hash`,
//! wrong input count, or failed verification returns `false`. It never traps
//! on caller-supplied data, so a bad proof costs the relayer gas but cannot
//! revert the `enter()` call. Only the constructor traps (on a malformed VK),
//! so a mis-deployed verifier is rejected at deploy time.
//!
//! [`NethermindEth/ultrahonk-rust-verifier`]: https://github.com/NethermindEth/ultrahonk-rust-verifier

extern crate alloc;

use soroban_sdk::{contract, contracterror, contractimpl, symbol_short, Bytes, BytesN, Env, Vec};
use ultrahonk_soroban_verifier::{UltraHonkVerifier, VkLoadError, PROOF_BYTES};

/// Number of 32-byte public-input words on the corridor wire — must equal
/// `corridor_types::abi::PI_LEN` (9: corridor_id, min_tier, now, nullifier,
/// disclosed_tag, issuer_id, min_cred_epoch, auditor_pubkey, auditor_blob).
/// Duplicated here instead of imported to keep the SDK-28 dependency graph
/// isolated; a compile-time assert would not catch a cross-repo drift anyway,
/// so this is checked in the Step-2 E2E test (a real proof verifies only when
/// the count and order match the circuit).
const PUB_INPUT_WORDS: u32 = 9;

#[contracterror]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// VK byte slice does not match the expected exact length (1760 bytes).
    VkInvalidLength = 1,
    /// VK header contains out-of-range structural parameters.
    VkInvalidParameters = 2,
    /// A VK G1 commitment was malformed (coordinate out of range or not on
    /// the BN254 curve).
    VkInvalidPoint = 3,
    /// Constructor has already been called; VK is immutable.
    AlreadyInitialized = 4,
    /// No VK has been stored in contract instance storage.
    VkNotSet = 5,
}

fn key_vk() -> soroban_sdk::Symbol {
    symbol_short!("vk")
}

fn key_vk_hash() -> soroban_sdk::Symbol {
    symbol_short!("vk_hash")
}

/// Concatenate the 32-byte public-input words into one `Bytes`, preserving
/// their `PI_*` order (the circuit proves them in exactly this order). The
/// core verifies against this contiguous encoding: it is what bb appends the
/// pairing-point object to when absorbing public inputs.
fn concat_public_inputs(env: &Env, words: &Vec<BytesN<32>>) -> Bytes {
    let mut out = Bytes::new(env);
    for i in 0..words.len() {
        // Length-checked by the caller; unreachable for `verify`.
        let w: BytesN<32> = words.get(i).expect("public input word in range");
        out.extend_from_array(&w.to_array());
    }
    out
}

/// The shared fail-closed body of `verify`. Every early exit is a `false`.
fn verify_impl(
    env: &Env,
    vk_hash: BytesN<32>,
    proof: &Bytes,
    public_inputs: &Vec<BytesN<32>>,
) -> bool {
    // Policy pin first (cheap): only proofs against the exact VK the policy
    // names verify.
    let stored: BytesN<32> = match env.storage().instance().get(&key_vk_hash()) {
        Some(h) => h,
        None => return false,
    };
    if vk_hash != stored {
        return false;
    }
    let vk_bytes: Bytes = match env.storage().instance().get(&key_vk()) {
        Some(vk) => vk,
        None => return false,
    };
    // Wire-level guards ahead of the core (cheap, no host crypto): the core
    // re-checks proof length, but the input-word count is corridor-specific.
    if public_inputs.len() != PUB_INPUT_WORDS {
        return false;
    }
    if proof.len() as usize != PROOF_BYTES {
        return false;
    }
    // Parse + validate the stored VK (structural params, curve membership).
    let verifier = match UltraHonkVerifier::new(env, &vk_bytes) {
        Ok(v) => v,
        Err(_) => return false,
    };
    // Full pipeline: transcript → sumcheck → Shplemini → pairing check.
    let pi_bytes = concat_public_inputs(env, public_inputs);
    verifier.verify(proof, &pi_bytes).is_ok()
}

#[contract]
pub struct UltrahonkVerifier;

#[contractimpl]
impl UltrahonkVerifier {
    /// Deploy with the serialized verification key (1760 bytes, bb 0.87.0
    /// `UltraKeccakFlavor` layout). Its SHA-256 is what policies pin via
    /// `vk_hash`. The VK is validated by parsing before storage, so a
    /// malformed VK aborts the deployment instead of poisoning storage.
    pub fn __constructor(env: Env, vk: Bytes) -> Result<(), Error> {
        if env.storage().instance().has(&key_vk()) {
            return Err(Error::AlreadyInitialized);
        }
        UltraHonkVerifier::new(&env, &vk).map_err(|e| match e {
            VkLoadError::WrongLength => Error::VkInvalidLength,
            VkLoadError::InvalidParameters => Error::VkInvalidParameters,
            VkLoadError::InvalidPoint => Error::VkInvalidPoint,
        })?;
        let hash = env.crypto().sha256(&vk).to_bytes();
        env.storage().instance().set(&key_vk(), &vk);
        env.storage().instance().set(&key_vk_hash(), &hash);
        Ok(())
    }

    /// The pinned VK hash — set this (and this contract's address) on the
    /// corridor policy.
    pub fn vk_hash(env: Env) -> Result<BytesN<32>, Error> {
        env.storage()
            .instance()
            .get(&key_vk_hash())
            .ok_or(Error::VkNotSet)
    }

    /// The stored VK bytes, for independent audit against the known-good
    /// circuit. Do not trust the contract address alone (see trust model).
    pub fn vk_bytes(env: Env) -> Result<Bytes, Error> {
        env.storage()
            .instance()
            .get(&key_vk())
            .ok_or(Error::VkNotSet)
    }

    /// Verify an UltraHonk proof for the corridor ABI. Fail-closed: returns
    /// `false` on any deviation, never traps on caller input.
    pub fn verify(
        env: Env,
        vk_hash: BytesN<32>,
        proof: Bytes,
        public_inputs: Vec<BytesN<32>>,
    ) -> bool {
        verify_impl(&env, vk_hash, &proof, &public_inputs)
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::{vec, Env};

    /// A structurally valid 1760-byte VK: header with `log_circuit_size = 1`,
    /// `public_inputs_size = 17` (→ 1 user public input after the 16-word
    /// pairing-point object), all 27 commitments set to the point at infinity
    /// (encoded as 64 zero bytes, which the core accepts). Good enough for
    /// the adapter's *guard* tests — the cryptographic accept/reject path is
    /// exercised by the Step-2 E2E test with real circuit artifacts.
    fn synthetic_vk(env: &Env) -> Bytes {
        let mut b = [0u8; 1760];
        b[7] = 2; // circuit_size = 1 << log_circuit_size
        b[15] = 1; // log_circuit_size = 1
        b[23] = 17; // public_inputs_size = 16 (pairing object) + 1 user
        b[31] = 2; // pub_inputs_offset = circuit_size
        Bytes::from_array(env, &b)
    }

    fn deploy(env: &Env) -> UltrahonkVerifierClient<'_> {
        let id = env.register(UltrahonkVerifier, (synthetic_vk(env),));
        UltrahonkVerifierClient::new(env, &id)
    }

    /// A malformed VK must abort the deployment, not poison storage.
    #[test]
    #[should_panic]
    fn constructor_rejects_a_malformed_vk() {
        let env = Env::default();
        // 64 bytes: right shape for a mistake, wrong length for a VK.
        env.register(UltrahonkVerifier, (Bytes::from_array(&env, &[1u8; 64]),));
    }

    #[test]
    fn rejects_a_wrong_vk_hash_before_any_cryptographic_work() {
        let env = Env::default();
        let client = deploy(&env);
        let bad = BytesN::from_array(&env, &[7u8; 32]);
        let proof = Bytes::from_array(&env, &[0u8; 8]);
        let pi = vec![&env, BytesN::from_array(&env, &[0u8; 32])];
        assert!(!client.verify(&bad, &proof, &pi));
    }

    #[test]
    fn rejects_a_wrong_length_proof() {
        let env = Env::default();
        let client = deploy(&env);
        let hash = client.vk_hash();
        let short_proof = Bytes::from_array(&env, &[0u8; PROOF_BYTES - 32]);
        let mut pi: Vec<BytesN<32>> = Vec::new(&env);
        let mut i = 0;
        while i < PUB_INPUT_WORDS {
            pi.push_back(BytesN::from_array(&env, &[0u8; 32]));
            i += 1;
        }
        assert!(!client.verify(&hash, &short_proof, &pi));
    }

    /// Deep guard test: a syntactically clean 14 592-byte proof and 9
    /// canonical input words still verify to `false`, because the stored VK
    /// expects a different public-input count. The core rejects after proof
    /// parse + padding validation, at the count check.
    #[test]
    fn rejects_when_public_input_count_does_not_match_the_vk() {
        let env = Env::default();
        let client = deploy(&env);
        let hash = client.vk_hash();
        let client = UltrahonkVerifierClient::new(
            &env,
            &env.register(UltrahonkVerifier, (synthetic_vk(&env),)),
        );
        let proof = Bytes::from_array(&env, &[0u8; PROOF_BYTES]);
        let mut pi: Vec<BytesN<32>> = Vec::new(&env);
        let mut i = 0;
        while i < PUB_INPUT_WORDS {
            pi.push_back(BytesN::from_array(&env, &[0u8; 32]));
            i += 1;
        }
        assert!(!client.verify(&hash, &proof, &pi));
    }

    #[test]
    fn vk_bytes_returns_the_stored_key() {
        let env = Env::default();
        let id = env.register(UltrahonkVerifier, (synthetic_vk(&env),));
        let client = UltrahonkVerifierClient::new(&env, &id);
        assert_eq!(client.vk_bytes(), synthetic_vk(&env));
        // And the pinned hash is the SHA-256 of those bytes.
        assert_eq!(
            client.vk_hash(),
            env.crypto().sha256(&synthetic_vk(&env)).to_bytes()
        );
    }
}
