#![cfg(test)]
extern crate std;

use crate::{CorridorRegistry, CorridorRegistryClient};
use corridor_types::{CorridorPolicy, Error};
use soroban_sdk::{testutils::Address as _, vec, Address, BytesN, Env};

fn zero32(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0u8; 32])
}

fn sample_policy(env: &Env, operator: &Address, verifier: &Address) -> CorridorPolicy {
    CorridorPolicy {
        operator: operator.clone(),
        accepted_issuers: vec![env, BytesN::from_array(env, &[7u8; 32])],
        min_tier: 2,
        required_disclosures: 0,
        credential_root: zero32(env),
        revocation_root: zero32(env),
        root_epoch: 0,
        verifier: verifier.clone(),
        vk_hash: BytesN::from_array(env, &[9u8; 32]),
        now_tolerance_secs: 300,
        paused: false,
    }
}

struct W {
    env: Env,
    registry: CorridorRegistryClient<'static>,
    admin: Address,
    operator: Address,
    verifier: Address,
    relayer: Address,
    cid: BytesN<32>,
}

fn setup() -> W {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let id = env.register(CorridorRegistry, (admin.clone(),));
    let registry = CorridorRegistryClient::new(&env, &id);
    let relayer = Address::generate(&env);
    registry.set_relayer(&relayer, &true);
    W {
        env: env.clone(),
        registry,
        admin,
        operator: Address::generate(&env),
        verifier: Address::generate(&env),
        relayer,
        cid: BytesN::from_array(&env, &[1u8; 32]),
    }
}

fn register(w: &W) {
    w.registry
        .register(&w.cid, &sample_policy(&w.env, &w.operator, &w.verifier));
}

#[test]
fn register_and_read_back() {
    let w = setup();
    register(&w);
    let got = w.registry.get_policy(&w.cid);
    assert_eq!(got.min_tier, 2);
    assert_eq!(got.root_epoch, 0);
    assert!(!got.paused);
}

#[test]
fn double_register_rejected() {
    let w = setup();
    register(&w);
    let policy = sample_policy(&w.env, &w.operator, &w.verifier);
    let err = w
        .registry
        .try_register(&w.cid, &policy)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::PolicyExists);
}

#[test]
fn post_root_advances_epoch() {
    let w = setup();
    register(&w);
    let root = BytesN::from_array(&w.env, &[42u8; 32]);
    w.registry
        .post_root(&w.relayer, &w.cid, &root, &zero32(&w.env), &1);
    let got = w.registry.get_policy(&w.cid);
    assert_eq!(got.root_epoch, 1);
    assert_eq!(got.credential_root, root);
}

#[test]
fn post_root_rejects_a_non_allowlisted_relayer() {
    let w = setup();
    register(&w);
    let stranger = Address::generate(&w.env);
    let err = w
        .registry
        .try_post_root(&stranger, &w.cid, &zero32(&w.env), &zero32(&w.env), &1)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::RelayerNotAllowed);
}

#[test]
fn admin_can_revoke_a_relayer() {
    let w = setup();
    register(&w);
    assert!(w.registry.is_relayer(&w.relayer));
    w.registry.set_relayer(&w.relayer, &false);
    assert!(!w.registry.is_relayer(&w.relayer));
    let err = w
        .registry
        .try_post_root(&w.relayer, &w.cid, &zero32(&w.env), &zero32(&w.env), &1)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::RelayerNotAllowed);
}

#[test]
fn post_root_epoch_cannot_regress() {
    let w = setup();
    register(&w);
    w.registry
        .post_root(&w.relayer, &w.cid, &zero32(&w.env), &zero32(&w.env), &5);
    let err = w
        .registry
        .try_post_root(&w.relayer, &w.cid, &zero32(&w.env), &zero32(&w.env), &5)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::RootEpochRegression);
}

#[test]
fn pause_toggles() {
    let w = setup();
    register(&w);
    w.registry.set_paused(&w.cid, &true);
    assert!(w.registry.get_policy(&w.cid).paused);
    w.registry.set_paused(&w.cid, &false);
    assert!(!w.registry.get_policy(&w.cid).paused);
}

#[test]
fn admin_transfer_is_two_step() {
    let w = setup();
    assert_eq!(w.registry.admin(), w.admin);
    let new_admin = Address::generate(&w.env);

    // nominate — role does not move yet
    w.registry.propose_admin(&new_admin);
    assert_eq!(w.registry.admin(), w.admin);

    // accept — now it moves
    w.registry.accept_admin();
    assert_eq!(w.registry.admin(), new_admin);
}

#[test]
fn accept_admin_without_a_proposal_fails() {
    let w = setup();
    let err = w.registry.try_accept_admin().err().unwrap().unwrap();
    assert_eq!(err, Error::NoPendingAdmin);
}

#[test]
fn update_policy_keeps_the_operator_and_roots() {
    let w = setup();
    register(&w);
    let root = BytesN::from_array(&w.env, &[42u8; 32]);
    w.registry
        .post_root(&w.relayer, &w.cid, &root, &zero32(&w.env), &3);

    // try to change everything, including the operator and roots
    let mut evil = sample_policy(&w.env, &Address::generate(&w.env), &w.verifier);
    evil.min_tier = 4;
    w.registry.update_policy(&w.cid, &evil);

    let got = w.registry.get_policy(&w.cid);
    assert_eq!(got.min_tier, 4); // operator-controlled field changed
    assert_eq!(got.operator, w.operator); // operator preserved
    assert_eq!(got.credential_root, root); // root preserved
    assert_eq!(got.root_epoch, 3); // epoch preserved
}
