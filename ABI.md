# Corridor public-input ABI

This repo is the **source of truth** for the layout of the ZK proof's public
inputs. The Noir circuit
([`corridor-circuits`](https://github.com/Sconce-Labs/corridor-circuits)) must
emit them in exactly this order; the SDK
([`corridor-sdk`](https://github.com/Sconce-Labs/corridor-sdk)) must assemble
them the same way. A change here is a coordinated change across all three repos.

Canonical definition: `crates/corridor_types/src/lib.rs` (`PI_*` constants and
`PublicInputs::decode`).

## Layout

`public_inputs` is a `Vec<BytesN<32>>` of length **9**. Each entry is a 32-byte
big-endian word.

| idx | const | field | encoding |
|----:|-------|-------|----------|
| 0 | `PI_CREDENTIAL_ROOT` | `credential_root` | 32-byte field element |
| 1 | `PI_REVOCATION_ROOT` | `revocation_root` | 32-byte field element |
| 2 | `PI_CORRIDOR_ID` | `corridor_id` | 32-byte id |
| 3 | `PI_MIN_TIER` | `min_tier` | `u32` in the low 4 bytes |
| 4 | `PI_NOW` | `now` | `u64` in the low 8 bytes |
| 5 | `PI_NULLIFIER` | `nullifier` | 32-byte field element, `Poseidon2(holder_secret, corridor_id)` |
| 6 | `PI_DISCLOSED_TAG` | `disclosed_tag` | `u32` in the low 4 bytes, `< 16` |
| 7 | `PI_ISSUER_ID` | `issuer_id` | 32-byte id |
| 8 | `PI_AUDITOR_BLOB` | `auditor_blob` | 32-byte commitment/ciphertext handle |

## Contract checks (in `corridor_attestation::enter`)

Bound against the corridor's on-chain `CorridorPolicy`:

- `credential_root` and `revocation_root` must equal the policy's synced roots
- `corridor_id` must equal the call's `corridor_id`
- `min_tier` must equal the policy's `min_tier`
- `issuer_id` must be in the policy's `accepted_issuers`
- `|ledger_timestamp - now|` must be `<= policy.now_tolerance_secs`
- the proof must verify against `policy.verifier` with `policy.vk_hash`
- `(corridor_id, nullifier)` must not already be recorded

## Hash conformance

`Poseidon2` must produce identical output across the Noir circuit, the SDK's
witness builder, and any on-chain hashing on Stellar — otherwise the Merkle
roots computed on Midnight, in the circuit, and checked on Stellar stop
agreeing.

**Pinned vector:** `poseidon2([1, 2]) == 0x038682aa1cb5ae4e0a3f13da432a95c77c5c111f6f030faf9cad641ce1ed7383`

| Implementation | Source | Status |
|----------------|--------|--------|
| Noir circuit | `noir-lang/poseidon` v0.3.0 | ✅ asserted in `corridor-circuits` |
| SDK witness builder | `@zkpassport/poseidon2` | ✅ asserted in `corridor-sdk` — matches |
| Soroban | `stellar/rs-soroban-poseidon` (`poseidon2_hash`, "matches noir's implementation") | ✅ asserted in `crates/poseidon_conformance` |

The Midnight side (`persistentHash` / `MerkleTree` hashing in `corridor.compact`)
is **not yet** on this list — M4 must confirm the Compact tree hash matches the
same vector before credentials issued on Midnight can be proven against on
Stellar.
