//! Typed events emitted by `corridor_registry`. Replaces the deprecated
//! `env.events().publish((symbol, ..), data)` calls.

use soroban_sdk::{contractevent, Address, BytesN};

/// A corridor was registered.
#[contractevent]
#[derive(Clone, Debug)]
pub struct Registered {
    #[topic]
    pub corridor_id: BytesN<32>,
    #[topic]
    pub operator: Address,
    pub min_tier: u32,
}

/// Operator-controlled fields of a corridor policy were updated.
#[contractevent]
#[derive(Clone, Debug)]
pub struct PolicyUpdated {
    #[topic]
    pub corridor_id: BytesN<32>,
    pub min_tier: u32,
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

/// A relayer posted a fresh Midnight root.
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
