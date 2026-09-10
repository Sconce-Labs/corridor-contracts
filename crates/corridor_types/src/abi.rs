//! The public-input layout — the ABI seam between the Noir circuit
//! (`corridor-circuits`), the SDK (`corridor-sdk`), and this contract.
//! Changing an index means changing all three plus `ABI.md`.
//!
//! Option B (issuer-signed statements): no credential/revocation roots.

use crate::errors::Error;
use soroban_sdk::{contracttype, BytesN, Env, Vec};

pub const PI_LEN: u32 = 9;
pub const PI_CORRIDOR_ID: u32 = 0;
pub const PI_MIN_TIER: u32 = 1;
pub const PI_NOW: u32 = 2;
pub const PI_NULLIFIER: u32 = 3;
pub const PI_DISCLOSED_TAG: u32 = 4;
pub const PI_ISSUER_ID: u32 = 5;
pub const PI_MIN_CRED_EPOCH: u32 = 6;
pub const PI_AUDITOR_PUBKEY: u32 = 7;
pub const PI_AUDITOR_BLOB: u32 = 8;

/// Typed view over the raw field-element vector the verifier consumes.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicInputs {
    pub corridor_id: BytesN<32>,
    pub min_tier: u32,
    pub now: u64,
    pub nullifier: BytesN<32>,
    pub disclosed_tag: u32,
    /// `Poseidon2(issuer_pk.x, issuer_pk.y)` — matched against the policy's
    /// `accepted_issuers`.
    pub issuer_id: BytesN<32>,
    /// Bulk-revocation floor — must equal `policy.min_cred_epoch`.
    pub min_cred_epoch: u64,
    /// Auditor key the blob binds to — must equal `policy.auditor_pubkey`.
    pub auditor_pubkey: BytesN<32>,
    pub auditor_blob: BytesN<32>,
}

impl PublicInputs {
    /// Decode the raw 32-byte words. Numeric fields are big-endian in the low
    /// bytes of their word; the high bytes **must be zero** (the circuit
    /// range-constrains these as `u32` / `u64`, so a non-canonical word is a
    /// malformed proof). `Error::BadPublicInputs` on a length mismatch or a
    /// non-canonical numeric word.
    pub fn decode(_env: &Env, raw: &Vec<BytesN<32>>) -> Result<PublicInputs, Error> {
        if raw.len() != PI_LEN {
            return Err(Error::BadPublicInputs);
        }
        Ok(PublicInputs {
            corridor_id: raw.get_unchecked(PI_CORRIDOR_ID),
            min_tier: word_to_u32(&raw.get_unchecked(PI_MIN_TIER))?,
            now: word_to_u64(&raw.get_unchecked(PI_NOW))?,
            nullifier: raw.get_unchecked(PI_NULLIFIER),
            disclosed_tag: word_to_u32(&raw.get_unchecked(PI_DISCLOSED_TAG))?,
            issuer_id: raw.get_unchecked(PI_ISSUER_ID),
            min_cred_epoch: word_to_u64(&raw.get_unchecked(PI_MIN_CRED_EPOCH))?,
            auditor_pubkey: raw.get_unchecked(PI_AUDITOR_PUBKEY),
            auditor_blob: raw.get_unchecked(PI_AUDITOR_BLOB),
        })
    }
}

/// Big-endian `u64` from the low 8 bytes of a word. `Err(BadPublicInputs)` if
/// any of the high 24 bytes is non-zero.
pub fn word_to_u64(w: &BytesN<32>) -> Result<u64, Error> {
    let a = w.to_array();
    let mut i = 0usize;
    while i < 24 {
        if a[i] != 0 {
            return Err(Error::BadPublicInputs);
        }
        i += 1;
    }
    let mut v = 0u64;
    while i < 32 {
        v = (v << 8) | a[i] as u64;
        i += 1;
    }
    Ok(v)
}

/// Big-endian `u32` from the low 4 bytes of a word. `Err(BadPublicInputs)` if
/// any of the high 28 bytes is non-zero.
pub fn word_to_u32(w: &BytesN<32>) -> Result<u32, Error> {
    let a = w.to_array();
    let mut i = 0usize;
    while i < 28 {
        if a[i] != 0 {
            return Err(Error::BadPublicInputs);
        }
        i += 1;
    }
    let mut v = 0u32;
    while i < 32 {
        v = (v << 8) | a[i] as u32;
        i += 1;
    }
    Ok(v)
}

/// Encode a `u64` into the low 8 bytes of a 32-byte word (test/relayer helper).
pub fn u64_to_word(env: &Env, v: u64) -> BytesN<32> {
    let mut a = [0u8; 32];
    let mut n = v;
    let mut i = 31usize;
    while i >= 24 {
        a[i] = (n & 0xff) as u8;
        n >>= 8;
        if i == 24 {
            break;
        }
        i -= 1;
    }
    BytesN::from_array(env, &a)
}

/// Encode a `u32` into the low 4 bytes of a 32-byte word (test/relayer helper).
pub fn u32_to_word(env: &Env, v: u32) -> BytesN<32> {
    let mut a = [0u8; 32];
    let mut n = v;
    let mut i = 31usize;
    while i >= 28 {
        a[i] = (n & 0xff) as u8;
        n >>= 8;
        if i == 28 {
            break;
        }
        i -= 1;
    }
    BytesN::from_array(env, &a)
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn word_round_trips() {
        let env = Env::default();
        for v in [0u64, 1, 255, 256, 1_000_000, u32::MAX as u64, u64::MAX] {
            assert_eq!(word_to_u64(&u64_to_word(&env, v)).unwrap(), v);
        }
        for v in [0u32, 1, 255, 65_535, u32::MAX] {
            assert_eq!(word_to_u32(&u32_to_word(&env, v)).unwrap(), v);
        }
    }

    #[test]
    fn word_rejects_non_canonical_high_bytes() {
        let env = Env::default();
        let mut a = [0u8; 32];
        a[23] = 1; // one bit above the u64 range
        assert_eq!(
            word_to_u64(&BytesN::from_array(&env, &a)).unwrap_err(),
            Error::BadPublicInputs
        );
        let mut b = [0u8; 32];
        b[27] = 1; // one bit above the u32 range
        assert_eq!(
            word_to_u32(&BytesN::from_array(&env, &b)).unwrap_err(),
            Error::BadPublicInputs
        );
    }

    #[test]
    fn decode_rejects_wrong_length() {
        let env = Env::default();
        let short: Vec<BytesN<32>> = Vec::new(&env);
        assert_eq!(
            PublicInputs::decode(&env, &short).unwrap_err(),
            Error::BadPublicInputs
        );
    }

    #[test]
    fn decode_rejects_a_non_canonical_min_tier_word() {
        let env = Env::default();
        let mut words: Vec<BytesN<32>> = Vec::new(&env);
        for i in 0..PI_LEN {
            let mut a = [0u8; 32];
            if i == PI_MIN_TIER {
                a[0] = 0xff; // garbage in the high bytes of the u32 field
            }
            words.push_back(BytesN::from_array(&env, &a));
        }
        assert_eq!(
            PublicInputs::decode(&env, &words).unwrap_err(),
            Error::BadPublicInputs
        );
    }
}
