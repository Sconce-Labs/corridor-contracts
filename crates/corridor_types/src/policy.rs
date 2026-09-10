use soroban_sdk::{contracttype, Address, BytesN, Vec};

/// The policy a corridor operator registers in `corridor_registry`.
///
/// `credential_root` / `revocation_root` / `root_epoch` are *not* set by the
/// operator directly — they are synced from Midnight via `post_root`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorridorPolicy {
    /// Who may pause / update this policy.
    pub operator: Address,
    /// Issuer ids this corridor accepts (checked against `PublicInputs::issuer_id`).
    pub accepted_issuers: Vec<BytesN<32>>,
    /// Minimum KYC tier required to pass (must equal the circuit's `min_tier`).
    pub min_tier: u32,
    /// Bitmask of disclosures the corridor requires the holder to include.
    pub required_disclosures: u32,
    /// Midnight credential-set Merkle root (synced).
    pub credential_root: BytesN<32>,
    /// Midnight revocation-set Merkle root (synced).
    pub revocation_root: BytesN<32>,
    /// Monotonic epoch of the synced roots.
    pub root_epoch: u64,
    /// Address of the ZK verifier contract this corridor trusts.
    pub verifier: Address,
    /// Hash of the verification key pinned for this corridor.
    pub vk_hash: BytesN<32>,
    /// The auditor public key passes must bind their `auditor_blob` to. All
    /// zero = this corridor has no auditor and the blob is a throwaway
    /// commitment. `enter` checks `public_inputs.auditor_pubkey` equals this.
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
