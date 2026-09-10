#![no_std]
//! The Soroban leg of Corridor's Poseidon2 hash-conformance gate.
//!
//! The Merkle roots on Midnight, inside the Noir circuit, and checked on
//! Stellar only agree if every implementation of Poseidon2 produces the same
//! output. `corridor-circuits` and `corridor-sdk` pin these vectors; this
//! asserts Soroban (`stellar/rs-soroban-poseidon`) matches. noir-lang/poseidon
//! v0.3.0 uses a t=4 (rate=3) sponge.

#[cfg(test)]
mod test {
    extern crate std;
    use soroban_poseidon::poseidon2_hash;
    use soroban_sdk::{crypto::bn254::Bn254Fr, vec, Env, Vec, U256};

    // 32-byte canonical forms, left-padded to 64 hex chars — identical strings
    // asserted in corridor-circuits and corridor-sdk.
    const H_1: &str = "168758332d5b3e2d13be8048c8011b454590e06c44bce7f702f09103eef5a373";
    const H_1_2: &str = "038682aa1cb5ae4e0a3f13da432a95c77c5c111f6f030faf9cad641ce1ed7383";
    const H_1_2_3: &str = "23864adb160dddf590f1d3303683ebcb914f828e2635f6e85a32f0a1aecd3dd8";
    const H_1_2_3_4_5: &str = "2247be7014a54d17342a7ef677f58d28877780d203860396967f5d0a18d259db";

    fn hex64(u: &U256) -> std::string::String {
        let mut s = std::string::String::new();
        for b in u.to_be_bytes().iter() {
            s.push_str(&std::format!("{:02x}", b));
        }
        while s.len() < 64 {
            s.insert(0, '0');
        }
        s
    }

    fn ints(env: &Env, xs: &[u32]) -> Vec<U256> {
        let mut v = vec![env];
        for x in xs {
            v.push_back(U256::from_u32(env, *x));
        }
        v
    }

    fn h(env: &Env, xs: &[u32]) -> std::string::String {
        hex64(&poseidon2_hash::<4, Bn254Fr>(env, &ints(env, xs)))
    }

    #[test]
    fn poseidon2_matches_the_pinned_vectors() {
        let env = Env::default();
        assert_eq!(h(&env, &[1]), H_1);
        assert_eq!(h(&env, &[1, 2]), H_1_2);
        assert_eq!(h(&env, &[1, 2, 3]), H_1_2_3);
        assert_eq!(h(&env, &[1, 2, 3, 4, 5]), H_1_2_3_4_5);
    }

    #[test]
    fn poseidon2_is_deterministic_and_order_sensitive() {
        let env = Env::default();
        assert_eq!(h(&env, &[3, 4]), h(&env, &[3, 4]));
        assert_ne!(h(&env, &[3, 4]), h(&env, &[4, 3]));
    }
}
