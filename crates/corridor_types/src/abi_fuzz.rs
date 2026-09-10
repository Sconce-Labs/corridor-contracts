//! Randomised tests for the public-input decoder. Host-only. Uses a tiny
//! xorshift PRNG so there is no `getrandom` / dev-dependency.

#![cfg(test)]
extern crate std;

use crate::abi::{u32_to_word, u64_to_word, word_to_u32, word_to_u64, PublicInputs, PI_LEN};
use crate::errors::Error;
use soroban_sdk::{BytesN, Env, Vec};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn words(env: &Env, raw: &[[u8; 32]]) -> Vec<BytesN<32>> {
    let mut v = Vec::new(env);
    for w in raw {
        v.push_back(BytesN::from_array(env, w));
    }
    v
}

#[test]
fn u64_word_round_trips_over_many_values() {
    let env = Env::default();
    let mut rng = Rng(0x9E3779B97F4A7C15);
    for _ in 0..2_000 {
        let v = rng.next();
        assert_eq!(word_to_u64(&u64_to_word(&env, v)), v);
    }
    for v in [0u64, 1, u64::MAX] {
        assert_eq!(word_to_u64(&u64_to_word(&env, v)), v);
    }
}

#[test]
fn u32_word_round_trips_over_many_values() {
    let env = Env::default();
    let mut rng = Rng(0xDEADBEEF12345678);
    for _ in 0..2_000 {
        let v = rng.next() as u32;
        assert_eq!(word_to_u32(&u32_to_word(&env, v)), v);
    }
}

#[test]
fn decode_never_panics_and_matches_length() {
    let env = Env::default();
    for len in 0usize..20 {
        let raw: std::vec::Vec<[u8; 32]> = std::vec![[0u8; 32]; len];
        let out = PublicInputs::decode(&env, &words(&env, &raw));
        if len as u32 == PI_LEN {
            assert!(out.is_ok());
        } else {
            assert_eq!(out.unwrap_err(), Error::BadPublicInputs);
        }
    }
}

#[test]
fn decode_preserves_numeric_fields_over_random_words() {
    let env = Env::default();
    let mut rng = Rng(0x1234_5678_9ABC_DEF0);
    for _ in 0..500 {
        let min_tier = rng.next() as u32;
        let now = rng.next();
        let tag = rng.next() as u32;
        let mut raw = [[0u8; 32]; PI_LEN as usize];
        raw[3] = u32_to_word(&env, min_tier).to_array();
        raw[4] = u64_to_word(&env, now).to_array();
        raw[6] = u32_to_word(&env, tag).to_array();
        let pi = PublicInputs::decode(&env, &words(&env, &raw)).unwrap();
        assert_eq!(pi.min_tier, min_tier);
        assert_eq!(pi.now, now);
        assert_eq!(pi.disclosed_tag, tag);
    }
}
