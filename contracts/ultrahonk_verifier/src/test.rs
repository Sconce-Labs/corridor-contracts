extern crate std;
use super::*;
use soroban_sdk::{vec, Bytes, BytesN, Env, Vec};

const VK_RAW: &[u8] = include_bytes!("../test_fixtures/vk");
const PROOF_RAW: &[u8] = include_bytes!("../test_fixtures/proof");
const PUBLIC_INPUTS_RAW: &[u8] = include_bytes!("../test_fixtures/public_inputs");

fn load_public_inputs(env: &Env) -> Vec<BytesN<32>> {
    let mut v = Vec::new(env);
    let (chunks, _) = PUBLIC_INPUTS_RAW.as_chunks::<32>();
    for chunk in chunks {
        v.push_back(BytesN::from_array(env, chunk));
    }
    v
}

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
    // right vk_hash → still false because dummy vk cannot parse
    assert!(!client.verify(&good, &proof, &pi));
}

#[test]
fn verifies_valid_ultrahonk_proof_with_real_vk() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let vk = Bytes::from_slice(&env, VK_RAW);
    let id = env.register(UltrahonkVerifier, (vk,));
    let client = UltrahonkVerifierClient::new(&env, &id);

    let vk_hash = client.vk_hash();
    let expected_hash = env
        .crypto()
        .sha256(&Bytes::from_slice(&env, VK_RAW))
        .to_bytes();
    assert_eq!(vk_hash, expected_hash);

    let proof = Bytes::from_slice(&env, PROOF_RAW);
    let pi = load_public_inputs(&env);

    assert!(client.verify(&vk_hash, &proof, &pi));
}

#[test]
fn rejects_tampered_proof() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let vk = Bytes::from_slice(&env, VK_RAW);
    let id = env.register(UltrahonkVerifier, (vk,));
    let client = UltrahonkVerifierClient::new(&env, &id);
    let vk_hash = client.vk_hash();

    let mut tampered = PROOF_RAW.to_vec();
    tampered[100] ^= 0x01; // flip one bit
    let proof = Bytes::from_slice(&env, &tampered);
    let pi = load_public_inputs(&env);

    assert!(!client.verify(&vk_hash, &proof, &pi));
}

#[test]
fn rejects_tampered_public_inputs() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let vk = Bytes::from_slice(&env, VK_RAW);
    let id = env.register(UltrahonkVerifier, (vk,));
    let client = UltrahonkVerifierClient::new(&env, &id);
    let vk_hash = client.vk_hash();

    let proof = Bytes::from_slice(&env, PROOF_RAW);
    let mut pi = load_public_inputs(&env);

    // Mutate the first public input
    let mut first: [u8; 32] = pi.get(0).unwrap().to_array();
    first[0] ^= 0xff;
    pi.set(0, BytesN::from_array(&env, &first));

    assert!(!client.verify(&vk_hash, &proof, &pi));
}

#[test]
fn rejects_mismatched_vk_hash_even_with_valid_proof() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let vk = Bytes::from_slice(&env, VK_RAW);
    let id = env.register(UltrahonkVerifier, (vk,));
    let client = UltrahonkVerifierClient::new(&env, &id);

    let bad_vk_hash = BytesN::from_array(&env, &[0xde; 32]);
    let proof = Bytes::from_slice(&env, PROOF_RAW);
    let pi = load_public_inputs(&env);

    assert!(!client.verify(&bad_vk_hash, &proof, &pi));
}

#[test]
fn rejects_invalid_proof_length() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let vk = Bytes::from_slice(&env, VK_RAW);
    let id = env.register(UltrahonkVerifier, (vk,));
    let client = UltrahonkVerifierClient::new(&env, &id);
    let vk_hash = client.vk_hash();

    // Proof shorter than 14592 bytes
    let short_proof = Bytes::from_slice(&env, &PROOF_RAW[..1000]);
    let pi = load_public_inputs(&env);

    assert!(!client.verify(&vk_hash, &short_proof, &pi));
}
