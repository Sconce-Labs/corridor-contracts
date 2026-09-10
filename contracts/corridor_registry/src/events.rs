//! Typed events emitted by `corridor_registry`.

use soroban_sdk::{contractevent, Address, BytesN};

/// A corridor was registered.
#[contractevent]
#[derive(Clone, Debug)]
pub struct RegisteredEvent {
    #[topic]
    pub corridor_id: BytesN<32>,
    #[topic]
    pub operator: Address,
    pub min_tier: u32,
}

/// Operator-controlled fields of a corridor policy were updated. Emits the
/// security-relevant fields so downstream consumers can react to a verifier swap.
#[contractevent]
#[derive(Clone, Debug)]
pub struct PolicyUpdated {
    #[topic]
    pub corridor_id: BytesN<32>,
    pub min_tier: u32,
    pub required_disclosures: u32,
    pub verifier: Address,
    pub vk_hash: BytesN<32>,
    pub paused: bool,
}

/// A corridor was paused or unpaused.
#[contractevent]
#[derive(Clone, Debug)]
pub struct PausedSet {
    #[topic]
    pub corridor_id: BytesN<32>,
    pub paused: bool,
}

/// An allowlisted relayer posted a fresh Midnight root.
#[contractevent]
#[derive(Clone, Debug)]
pub struct RootPosted {
    #[topic]
    pub corridor_id: BytesN<32>,
    #[topic]
    pub relayer: Address,
    pub epoch: u64,
    pub credential_root: BytesN<32>,
    pub revocation_root: BytesN<32>,
}

/// The admin added or removed a relayer from the `post_root` allowlist.
#[contractevent]
#[derive(Clone, Debug)]
pub struct RelayerSet {
    #[topic]
    pub relayer: Address,
    pub allowed: bool,
}

/// The admin role moved to a new address (after propose → accept).
#[contractevent]
#[derive(Clone, Debug)]
pub struct AdminTransferred {
    #[topic]
    pub old: Address,
    #[topic]
    pub new: Address,
}
