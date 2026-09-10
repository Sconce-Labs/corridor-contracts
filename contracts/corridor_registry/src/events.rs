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
    pub min_cred_epoch: u64,
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

/// The operator raised a corridor's bulk-revocation floor.
#[contractevent]
#[derive(Clone, Debug)]
pub struct MinCredEpochSet {
    #[topic]
    pub corridor_id: BytesN<32>,
    pub min_cred_epoch: u64,
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
