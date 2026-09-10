use crate::policy::CorridorPolicy;
use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, Vec};

/// ZK verifier contract interface. `verifier_mock` implements this for tests
/// and staged rollout; the real UltraHonk verifier is a drop-in replacement
/// with the same signature. `corridor_attestation` never hard-codes a verifier
/// — it calls whatever address the policy names.
#[contractclient(name = "VerifierClient")]
pub trait Verifier {
    fn verify(env: Env, vk_hash: BytesN<32>, proof: Bytes, public_inputs: Vec<BytesN<32>>) -> bool;
}

/// The subset of `corridor_registry` that `corridor_attestation` calls.
#[contractclient(name = "RegistryClient")]
pub trait RegistryInterface {
    fn get_policy(env: Env, corridor_id: BytesN<32>) -> CorridorPolicy;
    fn post_root(
        env: Env,
        relayer: Address,
        corridor_id: BytesN<32>,
        credential_root: BytesN<32>,
        revocation_root: BytesN<32>,
        epoch: u64,
    );
}
