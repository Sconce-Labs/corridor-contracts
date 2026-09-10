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

`Poseidon2` must be parameter-identical across Midnight (`persistentHash` / tree
hashing), the Noir circuit, and Soroban's `poseidon2_permutation` host function.
See the conformance test tracked in the roadmap — this is a correctness gate,
not a nicety.
