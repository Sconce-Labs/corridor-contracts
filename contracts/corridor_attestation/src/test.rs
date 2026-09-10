#![cfg(test)]
extern crate std;

use crate::{CorridorAttestation, CorridorAttestationClient};
use corridor_registry::{CorridorRegistry, CorridorRegistryClient};
use corridor_types::{u32_to_word, u64_to_word, CorridorPolicy, Error, PI_LEN};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    vec, Address, Bytes, BytesN, Env, Vec,
};
use verifier_mock::{VerifierMock, VerifierMockClient};

const ISSUER: [u8; 32] = [7u8; 32];
const NULLIFIER: [u8; 32] = [3u8; 32];
const CORRIDOR: [u8; 32] = [4u8; 32];

struct World {
    env: Env,
    attestation: CorridorAttestationClient<'static>,
    registry: CorridorRegistryClient<'static>,
    verifier: VerifierMockClient<'static>,
    operator: Address,
    cid: BytesN<32>,
}

fn b32(env: &Env, a: [u8; 32]) -> BytesN<32> {
    BytesN::from_array(env, &a)
}

fn policy(env: &Env, operator: &Address, verifier: &Address) -> CorridorPolicy {
    CorridorPolicy {
        operator: operator.clone(),
        accepted_issuers: vec![env, b32(env, ISSUER)],
        min_tier: 2,
        required_disclosures: 0,
        min_cred_epoch: 1,
        verifier: verifier.clone(),
        vk_hash: b32(env, [9u8; 32]),
        auditor_pubkey: b32(env, [0u8; 32]),
        now_tolerance_secs: 300,
        paused: false,
    }
}

fn setup() -> World {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000_000);

    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let verifier_id = env.register(VerifierMock, ());
    let registry_id = env.register(CorridorRegistry, (admin,));
    let attestation_id = env.register(CorridorAttestation, (registry_id.clone(),));
    let registry = CorridorRegistryClient::new(&env, &registry_id);
    let cid = b32(&env, CORRIDOR);
    registry.register(&cid, &policy(&env, &operator, &verifier_id));

    World {
        env: env.clone(),
        attestation: CorridorAttestationClient::new(&env, &attestation_id),
        registry,
        verifier: VerifierMockClient::new(&env, &verifier_id),
        operator,
        cid,
    }
}

/// A well-formed public-input vector for the happy path (Option B layout).
fn good_inputs(env: &Env) -> Vec<BytesN<32>> {
    let mut v = Vec::new(env);
    v.push_back(b32(env, CORRIDOR)); //          0 corridor_id
    v.push_back(u32_to_word(env, 2)); //         1 min_tier
    v.push_back(u64_to_word(env, 1_000_000)); // 2 now
    v.push_back(b32(env, NULLIFIER)); //         3 nullifier
    v.push_back(u32_to_word(env, 1)); //         4 disclosed_tag
    v.push_back(b32(env, ISSUER)); //            5 issuer_id
    v.push_back(u64_to_word(env, 1)); //         6 min_cred_epoch
    v.push_back(b32(env, [0u8; 32])); //         7 auditor_pubkey
    v.push_back(b32(env, [0u8; 32])); //         8 auditor_blob
    assert_eq!(v.len(), PI_LEN);
    v
}

#[test]
fn happy_path_grants_a_pass() {
    let w = setup();
    let proof = Bytes::from_array(&w.env, &[0xaa; 8]);
    w.attestation.enter(&w.cid, &proof, &good_inputs(&w.env));

    assert!(w.attestation.is_cleared(&w.cid, &b32(&w.env, NULLIFIER)));
    assert_eq!(w.attestation.passes(&w.cid), 1);
    assert_eq!(w.verifier.calls(), 1);
    assert_eq!(
        w.attestation
            .pass_record(&w.cid, &b32(&w.env, NULLIFIER))
            .unwrap()
            .tag,
        1
    );
}

#[test]
fn a_second_pass_bumps_the_counter() {
    let w = setup();
    let env = &w.env;
    let proof = Bytes::from_array(env, &[0xaa; 8]);
    w.attestation.enter(&w.cid, &proof, &good_inputs(env));
    let mut v2 = good_inputs(env);
    v2.set(3, b32(env, [0x77; 32]));
    w.attestation.enter(&w.cid, &proof, &v2);
    assert_eq!(w.attestation.passes(&w.cid), 2);
}

#[test]
fn nullifier_cannot_be_reused() {
    let w = setup();
    let proof = Bytes::from_array(&w.env, &[0xaa; 8]);
    w.attestation.enter(&w.cid, &proof, &good_inputs(&w.env));
    let err = w
        .attestation
        .try_enter(&w.cid, &proof, &good_inputs(&w.env))
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::NullifierUsed);
    assert_eq!(w.attestation.passes(&w.cid), 1);
}

#[test]
fn rejects_when_verifier_says_no() {
    let w = setup();
    w.verifier.set_result(&false);
    let proof = Bytes::from_array(&w.env, &[0xaa; 8]);
    let err = w
        .attestation
        .try_enter(&w.cid, &proof, &good_inputs(&w.env))
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::ProofInvalid);
}

#[test]
fn rejects_unaccepted_issuer() {
    let w = setup();
    let env = &w.env;
    let mut v = good_inputs(env);
    v.set(5, b32(env, [0x11; 32]));
    let err = w
        .attestation
        .try_enter(&w.cid, &Bytes::from_array(env, &[0xaa; 8]), &v)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::IssuerNotAccepted);
}

#[test]
fn rejects_min_tier_mismatch() {
    let w = setup();
    let env = &w.env;
    let mut v = good_inputs(env);
    v.set(1, u32_to_word(env, 3)); // proof claims min_tier 3, policy is 2
    let err = w
        .attestation
        .try_enter(&w.cid, &Bytes::from_array(env, &[0xaa; 8]), &v)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::MinTierMismatch);
}

#[test]
fn rejects_a_stale_cred_epoch_binding() {
    let w = setup();
    let env = &w.env;
    w.registry.set_min_cred_epoch(&w.cid, &5);
    // proof still binds min_cred_epoch = 1
    let err = w
        .attestation
        .try_enter(
            &w.cid,
            &Bytes::from_array(env, &[0xaa; 8]),
            &good_inputs(env),
        )
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::CredEpochMismatch);
}

#[test]
fn rejects_wrong_auditor_pubkey() {
    let w = setup();
    let env = &w.env;
    let mut v = good_inputs(env);
    v.set(7, b32(env, [0xab; 32]));
    let err = w
        .attestation
        .try_enter(&w.cid, &Bytes::from_array(env, &[0xaa; 8]), &v)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::DisclosureMissing);
}

#[test]
fn rejects_clock_skew_beyond_tolerance() {
    let w = setup();
    let env = &w.env;
    let mut v = good_inputs(env);
    v.set(2, u64_to_word(env, 1_000_000 - 10_000));
    let err = w
        .attestation
        .try_enter(&w.cid, &Bytes::from_array(env, &[0xaa; 8]), &v)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::StaleProofTime);
}

#[test]
fn rejects_when_paused() {
    let w = setup();
    w.registry.set_paused(&w.cid, &true);
    let err = w
        .attestation
        .try_enter(
            &w.cid,
            &Bytes::from_array(&w.env, &[0xaa; 8]),
            &good_inputs(&w.env),
        )
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::PolicyPaused);
}

#[test]
fn rejects_bad_public_input_length() {
    let w = setup();
    let env = &w.env;
    let v: Vec<BytesN<32>> = vec![env, b32(env, CORRIDOR)];
    let err = w
        .attestation
        .try_enter(&w.cid, &Bytes::from_array(env, &[0xaa; 8]), &v)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::BadPublicInputs);
}

#[test]
fn different_corridors_have_independent_nullifiers() {
    let w = setup();
    let env = &w.env;
    let cid2 = b32(env, [0x99; 32]);
    w.registry
        .register(&cid2, &policy(env, &w.operator, &w.verifier.address));
    let proof = Bytes::from_array(env, &[0xaa; 8]);

    w.attestation.enter(&w.cid, &proof, &good_inputs(env));
    let mut v2 = good_inputs(env);
    v2.set(0, cid2.clone());
    w.attestation.enter(&cid2, &proof, &v2);

    assert!(w.attestation.is_cleared(&w.cid, &b32(env, NULLIFIER)));
    assert!(w.attestation.is_cleared(&cid2, &b32(env, NULLIFIER)));
}
