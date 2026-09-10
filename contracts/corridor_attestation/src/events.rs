//! Typed events for `corridor_attestation`.

use soroban_sdk::{contractevent, BytesN};

/// A holder entered a corridor: a pass was granted and the nullifier burned.
#[contractevent]
#[derive(Clone, Debug)]
pub struct PassGranted {
    #[topic]
    pub corridor_id: BytesN<32>,
    #[topic]
    pub nullifier: BytesN<32>,
    pub tag: u32,
    pub passes: u64,
}
