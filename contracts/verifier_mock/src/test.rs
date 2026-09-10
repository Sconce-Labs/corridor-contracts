#![cfg(test)]
extern crate std;

use crate::{VerifierMock, VerifierMockClient};
use soroban_sdk::{vec, Bytes, BytesN, Env, Vec};

fn setup() -> (Env, VerifierMockClient<'static>) {
    let env = Env::default();
    let id = env.register(VerifierMock, ());
    (env.clone(), VerifierMockClient::new(&env, &id))
}

fn args(env: &Env) -> (BytesN<32>, Bytes, Vec<BytesN<32>>) {
    (
        BytesN::from_array(env, &[0u8; 32]),
        Bytes::from_array(env, &[0u8; 4]),
        vec![env, BytesN::from_array(env, &[0u8; 32])],
    )
}

#[test]
fn defaults_to_true_and_counts_calls() {
    let (env, v) = setup();
    let (vk, proof, pi) = args(&env);
    assert_eq!(v.calls(), 0);
    assert!(v.verify(&vk, &proof, &pi));
    assert!(v.verify(&vk, &proof, &pi));
    assert_eq!(v.calls(), 2);
}

#[test]
fn set_result_false_makes_verify_reject() {
    let (env, v) = setup();
    let (vk, proof, pi) = args(&env);
    v.set_result(&false);
    assert!(!v.verify(&vk, &proof, &pi));
    v.set_result(&true);
    assert!(v.verify(&vk, &proof, &pi));
}
