#![no_std]
//! Shared types and cross-contract interfaces for Corridor on Stellar.
//!
//! The ABI seam between three moving parts:
//!   * the Noir circuit  (`corridor-circuits`) — produces `public_inputs`
//!   * `corridor_registry` — stores [`CorridorPolicy`] per corridor
//!   * `corridor_attestation` — decodes public inputs, calls a [`Verifier`],
//!     records a [`PassRecord`]
//!
//! Changing the public-input layout ([`abi`]) means changing the circuit, the
//! SDK, and `ABI.md` in the same coordinated PR.

mod abi;
mod errors;
mod interfaces;
mod policy;

pub use abi::{
    u32_to_word, u64_to_word, word_to_u32, word_to_u64, PublicInputs, PI_AUDITOR_BLOB,
    PI_CORRIDOR_ID, PI_CREDENTIAL_ROOT, PI_DISCLOSED_TAG, PI_ISSUER_ID, PI_LEN, PI_MIN_TIER,
    PI_NOW, PI_NULLIFIER, PI_REVOCATION_ROOT,
};
pub use errors::Error;
pub use interfaces::{RegistryClient, RegistryInterface, Verifier, VerifierClient};
pub use policy::{CorridorPolicy, PassRecord};
