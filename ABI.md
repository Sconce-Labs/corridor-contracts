# Corridor public-input ABI

This repo is the **source of truth** for the layout of the ZK proof's public
inputs. The Noir circuit
([`corridor-circuits`](https://github.com/Sconce-Labs/corridor-circuits)) must
emit them in exactly this order; the SDK
([`corridor-sdk`](https://github.com/Sconce-Labs/corridor-sdk)) must assemble
them the same way. A change here is a coordinated change across all three repos.

Canonical definition: `crates/corridor_types/src/abi.rs` (`PI_*` constants and
`PublicInputs::decode`).

## Layout (Option B — issuer-signed statements)

`public_inputs` is a `Vec<BytesN<32>>` of length **9**. Each entry is a 32-byte
big-endian word.

| idx | const | field | encoding |
|----:|-------|-------|----------|
| 0 | `PI_CORRIDOR_ID` | `corridor_id` | 32-byte id |
| 1 | `PI_MIN_TIER` | `min_tier` | `u32`, big-endian in the low 4 bytes; **high 28 bytes must be zero** |
| 2 | `PI_NOW` | `now` | `u64`, big-endian in the low 8 bytes; **high 24 bytes must be zero** |
| 3 | `PI_NULLIFIER` | `nullifier` | 32-byte field element, `Poseidon2(holder_secret, corridor_id)` |
| 4 | `PI_DISCLOSED_TAG` | `disclosed_tag` | `u32` in the low 4 bytes, `< 16`; **high 28 bytes must be zero**. A category label only — the circuit does **not** bind it to any attribute (audit R2-H1) |
| 5 | `PI_ISSUER_ID` | `issuer_id` | 32-byte id, `Poseidon2(issuer_pk.x, issuer_pk.y)` |
| 6 | `PI_MIN_CRED_EPOCH` | `min_cred_epoch` | `u64` in the low 8 bytes — the bulk-revocation floor; **high 24 bytes must be zero** |
| 7 | `PI_AUDITOR_PUBKEY` | `auditor_pubkey` | 32-byte key `< BN254_P` (it's a circuit `Field`) — must equal `policy.auditor_pubkey` (`0` = no auditor) |
| 8 | `PI_AUDITOR_BLOB` | `auditor_blob` | `Poseidon2(auditor_pubkey, tier, issuer_id, nullifier, auditor_nonce)` |

`PublicInputs::decode` returns `BadPublicInputs` on a length mismatch **or** a
non-canonical numeric word (non-zero high bytes on `min_tier` / `now` /
`disclosed_tag` / `min_cred_epoch`). The circuit already range-constrains these
as `u32` / `u64`, so this is defence in depth.

### What the circuit proves (private witness, not public)

`holder_secret, tier, expiry, cred_epoch, salt, issuer_pk_{x,y},
sig_{s,e}_{lo,hi}, auditor_nonce`. The circuit checks a **Grumpkin Schnorr
signature** (`noir-lang/schnorr` v0.4.0) by `issuer_pk` over
`Poseidon2([Poseidon2([holder_secret, salt]), tier, expiry, cred_epoch])`, then
`issuer_id == Poseidon2(pk.x, pk.y)`, `tier >= min_tier`, `expiry > now`,
`cred_epoch >= min_cred_epoch`, the nullifier, the tag bound, and the auditor
blob. There is no Merkle path — Option B has no accumulator (audit C1/C4, see
`corridor/docs/CREDENTIAL_ACCUMULATOR.md`).

## Contract checks (in `corridor_attestation::enter`)

Bound against the corridor's on-chain `CorridorPolicy`:

- `corridor_id` must equal the call's `corridor_id`
- `min_tier` must equal the policy's `min_tier`
- `issuer_id` must be in the policy's `accepted_issuers`
- `min_cred_epoch` must equal the policy's `min_cred_epoch` (`CredEpochMismatch` otherwise)
- `auditor_pubkey` must equal the policy's `auditor_pubkey`
- `|ledger_timestamp - now|` must be `<= policy.now_tolerance_secs`
- the proof must verify against `policy.verifier` with `policy.vk_hash`
- `(corridor_id, nullifier)` must not already be recorded

## Hash conformance

`Poseidon2` must produce identical output across the Noir circuit, the SDK's
witness builder, and any on-chain hashing on Stellar — otherwise the issuer_id,
nullifier and auditor_blob computed in the circuit and re-derived elsewhere stop
agreeing. The circuit uses **arity 2** (`holder_binding`, `issuer_id`,
`nullifier`), **arity 4** (the signed statement message) and **arity 5**
(`auditor_blob`); all three are pinned explicitly in
`corridor-circuits/src/conformance.nr`, `corridor-sdk/src/poseidon.test.ts`, and
`corridor-contracts/crates/poseidon_conformance`.

**Pinned vectors** (t=4 / rate-3 sponge, BN254):

| input | output (32-byte canonical) | used for |
|-------|---------------------------|----------|
| `[1]` | `0x168758332d5b3e2d13be8048c8011b454590e06c44bce7f702f09103eef5a373` | — |
| `[1,2]` | `0x038682aa1cb5ae4e0a3f13da432a95c77c5c111f6f030faf9cad641ce1ed7383` | holder_binding, issuer_id, nullifier |
| `[1,2,3]` | `0x23864adb160dddf590f1d3303683ebcb914f828e2635f6e85a32f0a1aecd3dd8` | — |
| `[1,2,3,4]` | `0x130bf204a32cac1f0ace56c78b731aa3809f06df2731ebcf6b3464a15788b1b9` | **statement message** |
| `[1,2,3,4,5]` | `0x2247be7014a54d17342a7ef677f58d28877780d203860396967f5d0a18d259db` | **auditor_blob** |

| Implementation | Source | Status |
|----------------|--------|--------|
| Noir circuit | `noir-lang/poseidon` v0.3.0 | ✅ `[1,2]`, `[1,2,3,4]`, `[1,2,3,4,5]` asserted in `corridor-circuits/src/conformance.nr` |
| SDK witness builder | `@zkpassport/poseidon2` | ✅ all five asserted in `corridor-sdk/src/poseidon.test.ts` |
| Soroban | `stellar/rs-soroban-poseidon` | ✅ all five asserted in `crates/poseidon_conformance` |

Under Option B, `corridor.compact` does no hash-critical work (it stores a plain
issuer directory), so Midnight is not on this list and does not need to be.

**Schnorr conformance:** `corridor-sdk/src/schnorr.ts` is checked against
`noir-lang/schnorr` v0.4.0's pinned test vector, and `nargo execute` in circuit
CI solves a fixture the SDK signed — so the SDK signer and the circuit verifier
provably agree.
