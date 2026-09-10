#![no_std]
//! The Soroban leg of Corridor's Poseidon2 hash-conformance gate.
//!
//! The Merkle roots on Midnight, inside the Noir circuit, and checked on
//! Stellar only agree if every implementation of Poseidon2 produces the same
//! output. `corridor-circuits` and `corridor-sdk` already pin
//! `poseidon2([1, 2]) == 0x038682…1ed7383`. This asserts Soroban does too,
//! via `stellar/rs-soroban-poseidon` (whose README states it matches Noir).

#[cfg(test)]
mod test {
    extern crate std;
    use soroban_poseidon::poseidon2_hash;
    use soroban_sdk::{crypto::bn254::Bn254Fr, vec, Env, U256};

    /// The 32-byte value `corridor-circuits` and `corridor-sdk` assert for
    /// `poseidon2([1, 2])`.
    const PINNED: &str = "038682aa1cb5ae4e0a3f13da432a95c77c5c111f6f030faf9cad641ce1ed7383";

    fn hex32(u: &U256) -> std::string::String {
        let bytes = u.to_be_bytes();
        let mut s = std::string::String::new();
        for b in bytes.iter() {
            s.push_str(&std::format!("{:02x}", b));
        }
        // trim to the minimal form the other two repos print (no leading 00 byte)
        s.trim_start_matches("00").into()
    }

    #[test]
    fn poseidon2_of_1_2_matches_the_pinned_vector() {
        let env = Env::default();
        let inputs = vec![&env, U256::from_u32(&env, 1), U256::from_u32(&env, 2)];
        // noir-lang/poseidon v0.3.0 uses a t=4 (rate=3) sponge.
        let h = poseidon2_hash::<4, Bn254Fr>(&env, &inputs);
        let got = hex32(&h);
        assert_eq!(got, PINNED, "Soroban Poseidon2 diverged from Noir/SDK");
    }
}
