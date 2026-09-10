use soroban_sdk::{contracttype, Address, BytesN, Vec};

/// The policy a corridor operator registers in `corridor_registry`.
///
/// Option B: no synced Midnight roots. Eligibility is proven against an
/// issuer's Grumpkin Schnorr signature; revocation is short credential expiry
/// plus `min_cred_epoch`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorridorPolicy {
    /// Who may pause / update this policy.
    pub operator: Address,
    /// Issuer ids this corridor accepts — `Poseidon2(issuer_pk.x, issuer_pk.y)`
    /// for each accepted issuer's Schnorr key. Checked against the proof's
    /// `issuer_id` public input.
    pub accepted_issuers: Vec<BytesN<32>>,
    /// Minimum KYC tier required to pass (must equal the circuit's `min_tier`).
    pub min_tier: u32,
    /// Bitmask of disclosures the corridor requires. Reserved for selective
    /// disclosure; currently only `auditor_pubkey != 0` is enforced.
    pub required_disclosures: u32,
    /// Bulk-revocation floor: a credential's `cred_epoch` must be `>=` this.
    /// The operator raises it (per issuer guidance) to invalidate everything
    /// issued before an epoch.
    pub min_cred_epoch: u64,
    /// Address of the ZK verifier contract this corridor trusts.
    pub verifier: Address,
    /// Hash of the verification key pinned for this corridor.
    pub vk_hash: BytesN<32>,
    /// The auditor public key passes must bind their `auditor_blob` to. All
    /// zero = no auditor. `enter` checks `public_inputs.auditor_pubkey` equals
    /// this.
    pub auditor_pubkey: BytesN<32>,
    /// Allowed skew between the proof's `now` and ledger time, in seconds.
    pub now_tolerance_secs: u64,
    /// Kill switch — when true, `enter` rejects every proof.
    pub paused: bool,
}

/// Written when a holder successfully enters a corridor. Keyed by
/// `(corridor_id, nullifier)` so it is unlinkable across corridors.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PassRecord {
    /// Small enum index the holder chose to disclose (not free text).
    pub tag: u32,
    /// Ledger sequence the pass was granted at.
    pub ledger: u32,
    /// Ledger timestamp the pass was granted at.
    pub timestamp: u64,
    /// Binding of `{tier, issuer_id}` to the nullifier under the auditor key.
    /// A hiding commitment in the MVP; real ciphertext after M7.
    pub auditor_blob: BytesN<32>,
}
