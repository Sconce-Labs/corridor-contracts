# corridor-contracts

[![CI](https://github.com/Sconce-Labs/corridor-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Sconce-Labs/corridor-contracts/actions/workflows/ci.yml)
[![Stellar testnet](https://img.shields.io/badge/Stellar-testnet-brightgreen)](./deployments/testnet.json)
[![soroban-sdk](https://img.shields.io/badge/soroban--sdk-25.3-blue)](https://docs.rs/soroban-sdk)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](./LICENSE)

The **Stellar side of [Corridor](https://github.com/Sconce-Labs/corridor)** — a
portable, zero-knowledge proof of eligibility for cross-border payments.

This workspace holds the Soroban contracts that a corridor operator deploys and
integrates against: the **policy registry**, the **attestation contract** that
binds a holder's ZK proof to the corridor policy, calls the policy's verifier,
and burns a per-corridor nullifier, and the **verifier** behind a stable
interface. It is the **source of truth for the public-input ABI**
([`ABI.md`](./ABI.md)) that the Noir circuit and the SDK must match.

> **The policy binding is live on testnet; the ZK verifier is a mock** that
> returns `true` until the real UltraHonk verifier lands (milestone **M3** —
> [#1](https://github.com/Sconce-Labs/corridor-contracts/issues/1)). Until then,
> `is_cleared` on testnet attests the policy binding and one-time use, **not**
> the cryptographic proof.

| | |
|---|---|
| **Product & architecture** | [Sconce-Labs/corridor](https://github.com/Sconce-Labs/corridor) · [`ARCHITECTURE.md`](https://github.com/Sconce-Labs/corridor/blob/main/ARCHITECTURE.md) |
| **Noir circuit** | [Sconce-Labs/corridor-circuits](https://github.com/Sconce-Labs/corridor-circuits) |
| **Client SDK** | [Sconce-Labs/corridor-sdk](https://github.com/Sconce-Labs/corridor-sdk) |
| **Public-input ABI** | [`ABI.md`](./ABI.md) |
| **Deployed addresses** | [`deployments/testnet.json`](./deployments/testnet.json) |

---

## How it fits together

```
                        corridor-circuits            corridor-sdk
                        (Noir → UltraHonk)      (buildWitness / issueCredential)
                               │                          │
                               │  proof + 9 public inputs  │
                               ▼                          ▼
┌──────────────────────────────────────────────────────────────────────┐
│  corridor_attestation.enter(corridor_id, proof, public_inputs)        │
│    1. load Policy(corridor_id) from corridor_registry, assert !paused │
│    2. bind public inputs ↔ policy                                     │
│         corridor_id · min_tier · issuer_id ∈ accepted_issuers ·       │
│         min_cred_epoch · auditor_pubkey · |now − ledger.time| ≤ tol   │
│    3. verifier.verify(vk_hash, proof, public_inputs)   ← Protocol 25  │
│    4. assert nullifier unused → store PassRecord → Passes += 1        │
│    5. emit PassGranted                                                │
└──────────────────────────────────────────────────────────────────────┘
                               │
          operator's payout contract ── is_cleared(corridor_id, nullifier)? ──▶ pay / deny
```

Eligibility is proven against an **issuer's Grumpkin Schnorr signature** over a
short-lived statement `{holder_binding, tier, expiry, cred_epoch}` — there is no
credential accumulator and no cross-chain state. Revocation is short `expiry`
plus a monotonic `min_cred_epoch` floor. See
[`docs/CREDENTIAL_ACCUMULATOR.md`](https://github.com/Sconce-Labs/corridor/blob/main/docs/CREDENTIAL_ACCUMULATOR.md)
for why the earlier design was dropped.

---

## Workspace

```
corridor-contracts/
├── Cargo.toml                         workspace · soroban-sdk 25.3 · panic=abort · lto
├── ABI.md                             ← source of truth for the PI_* public-input layout
├── crates/
│   ├── corridor_types/                shared types, errors, ABI codec, cross-contract interfaces
│   └── poseidon_conformance/          test-only: Soroban Poseidon2 == circuit == SDK vector
├── contracts/
│   ├── corridor_registry/             CorridorPolicy CRUD · set_min_cred_epoch · two-step admin
│   ├── corridor_attestation/          enter() · is_cleared() · nullifier ledger · events · TTL
│   ├── ultrahonk_verifier/            real UltraHonk verifier skeleton (M3)
│   └── verifier_mock/                 configurable pass/fail verifier (tests + staged rollout)
├── deployments/testnet.json           live addresses + smoke-test tx hashes
├── scripts/                           deploy_testnet.sh · demo.sh · gas.sh
└── Makefile                           common tasks
```

| Crate | Kind | Role |
|-------|------|------|
| `crates/corridor_types` | rlib | `CorridorPolicy`, `PassRecord`, `PublicInputs` (encode/decode), `Error` (stable numbers), `VerifierClient` / `RegistryInterface`, the `PI_*` constants. **Consumed by every contract.** |
| `crates/poseidon_conformance` | `#[cfg(test)]` bin | Pins `poseidon2([1,2]) == 0x038682…1ed7383` and three more vectors against `rs-soroban-poseidon`, so the on-chain hash provably matches Noir and the SDK. |
| `contracts/corridor_registry` | contract | Stores one `CorridorPolicy` per corridor. Operator-gated `register` / `update_policy` / `set_paused` / `set_min_cred_epoch` (monotonic); admin is two-step (`propose_admin` → `accept_admin`). |
| `contracts/corridor_attestation` | contract | `enter()` binds a proof to a policy, calls the verifier, burns `(corridor_id, nullifier)`, writes a `PassRecord`, bumps `Passes`. `is_cleared()` / `pass_record()` / `passes()` are the read side. Persistent entries are TTL-extended on every read. |
| `contracts/ultrahonk_verifier` | contract | The real Protocol-25 UltraHonk verifier — **skeleton only** (pins a `vk_hash`, `verify()` returns `false`). Wiring it is milestone **M3**. See its [README](./contracts/ultrahonk_verifier/README.md). |
| `contracts/verifier_mock` | contract | Implements `VerifierClient` with a settable answer. Used by every host test and by the testnet deployment until M3. **Never deploy to a corridor that guards real value.** |

---

## Public-input ABI

`public_inputs` is a `Vec<BytesN<32>>` of length **9** (big-endian words). Full
spec, encodings, and the contract checks are in [`ABI.md`](./ABI.md).

| idx | field | idx | field |
|----:|-------|----:|-------|
| 0 | `corridor_id` | 5 | `issuer_id` = `Poseidon2(pk.x, pk.y)` |
| 1 | `min_tier` | 6 | `min_cred_epoch` |
| 2 | `now` | 7 | `auditor_pubkey` |
| 3 | `nullifier` | 8 | `auditor_blob` |
| 4 | `disclosed_tag` | | |

> Changing this layout is a **coordinated PR** across `corridor-contracts`
> (+ `ABI.md`), `corridor-circuits`, and `corridor-sdk`.

---

## Build & test

**Prerequisites:** Rust stable, the `wasm32v1-none` target
(`rustup target add wasm32v1-none`), and — to deploy — the
[`stellar` CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli)
≥ 22.

```bash
cargo test --workspace --locked        # 30 host tests, no network
cargo fmt --all -- --check
cargo clippy --workspace --all-targets

# contract wasm
cargo build --locked --release --target wasm32v1-none \
  -p corridor-registry -p corridor-attestation \
  -p verifier-mock -p ultrahonk-verifier

# or just:
make test        # fmt + clippy + test
make build       # all four wasm artifacts
```

**Windows (GNU toolchain):** `.cargo/config.toml` adds
`-Wl,--exclude-all-symbols` to get past `ld`'s "export ordinal too large" on
soroban-sdk's dependency tree. Harmless on Linux CI. The first build is ~5 min.

---

## Continuous integration

[`.github/workflows/ci.yml`](./.github/workflows/ci.yml) runs on every push and
PR to `main`:

| Job | Steps | Blocking |
|-----|-------|----------|
| **`fmt + clippy + test + wasm`** | `cargo fmt --check` · `cargo clippy --workspace --all-targets` · `cargo test --workspace --locked` (30 tests) · release wasm build for all four contracts · uploads the `.wasm` artifacts | ✅ required for merge |
| **`cargo-deny (advisory)`** | `cargo deny check` — licenses, advisories, duplicate/wildcard deps | ⚠️ `continue-on-error` — informational |

`main` is protected: the `fmt + clippy + test + wasm` check must pass.
Dependabot ([`.github/dependabot.yml`](./.github/dependabot.yml)) watches Cargo
and Actions weekly.

---

## Testnet deployment

Option B ABI (9 public inputs), deployed 2026-09-10 —
[`deployments/testnet.json`](./deployments/testnet.json):

| Contract | Address |
|----------|---------|
| `corridor_registry` | [`CDGMQ24E6OIBZB3EKJN5TUA5POYE6D5FNBL2II6SRLTYF32TE4HIEXJ6`](https://stellar.expert/explorer/testnet/contract/CDGMQ24E6OIBZB3EKJN5TUA5POYE6D5FNBL2II6SRLTYF32TE4HIEXJ6) |
| `corridor_attestation` | [`CCHWKVRCEKPJHEXREP5SCZ4TEKNBFYEFYA2VR3SET5AMG4WOC76LDL4K`](https://stellar.expert/explorer/testnet/contract/CCHWKVRCEKPJHEXREP5SCZ4TEKNBFYEFYA2VR3SET5AMG4WOC76LDL4K) |
| `verifier_mock` (placeholder — M3) | [`CBN7N7AT7CPAA7MBIAULEBY3GIV7NNB3XPNEUJSIAHFIM5BJ7GIGK46Y`](https://stellar.expert/explorer/testnet/contract/CBN7N7AT7CPAA7MBIAULEBY3GIV7NNB3XPNEUJSIAHFIM5BJ7GIGK46Y) |

Smoke-verified on testnet: `register → enter` (PassGranted, tag 1, passes 1)
`→ is_cleared == true`; replay rejected with `NullifierUsed` (Error #12).

Deploy your own stack:

```bash
stellar keys generate corridor --network testnet --fund
./scripts/deploy_testnet.sh          # builds wasm, deploys all three, prints addresses
VERIFIER=… REGISTRY=… ATTESTATION=… ./scripts/demo.sh    # end-to-end smoke
```

---

## Integrating a corridor

Three calls — that is the whole surface.

**1 · Register a policy** (operator key):

```rust
corridor_registry.register(corridor_id, CorridorPolicy {
    operator,
    accepted_issuers,        // Vec<BytesN<32>> of Poseidon2(pk.x, pk.y) ids
    min_tier,                // u32
    required_disclosures,    // u32 (reserved)
    min_cred_epoch,          // u64 — bulk-revocation floor
    verifier,                // Address of a VerifierClient
    vk_hash,                 // BytesN<32> — pins the verification key
    auditor_pubkey,          // BytesN<32> — 0 = no auditor
    now_tolerance_secs,      // u64 — proof-time skew window (e.g. 300)
    paused: false,
});
```

**2 · Track revocation** — when an accepted issuer bumps its credential epoch,
raise your floor (monotonic; rejects a lower value):

```rust
corridor_registry.set_min_cred_epoch(corridor_id, new_epoch);
```

**3 · Gate the payout** — in your existing payout contract, before releasing
funds:

```rust
if !corridor_attestation.is_cleared(corridor_id, nullifier) {
    panic!("recipient not cleared for this corridor");
}
```

The holder's app submits `enter()` (via a fee-sponsoring relayer, M6) and hands
the operator the `nullifier` alongside the payment request.

---

## Events

All events use `#[contractevent]` (`events.rs` in each contract).

| Contract | Event | Topics | Data |
|----------|-------|--------|------|
| registry | `Registered` | `corridor_id`, `operator` | `min_tier` |
| registry | `PolicyUpdated` | `corridor_id` | `min_tier`, `required_disclosures`, `min_cred_epoch`, `verifier`, `vk_hash`, `paused` |
| registry | `PausedSet` | `corridor_id` | `paused` |
| registry | `MinCredEpochSet` | `corridor_id` | `min_cred_epoch` |
| registry | `AdminTransferred` | `old`, `new` | — |
| attestation | `PassGranted` | `corridor_id`, `nullifier` | `tag`, `passes` |

---

## Errors

Stable numbers (`crates/corridor_types/src/errors.rs`), surfaced as
`Error(Contract, #N)`:

| # | Name | Meaning |
|--:|------|---------|
| 3 | `NotAuthorized` | caller is not the operator / admin |
| 4 | `PolicyNotFound` | no policy for this `corridor_id` |
| 5 | `PolicyPaused` | the policy's kill switch is on |
| 6 | `PolicyExists` | `register` called twice for one corridor |
| 8 | `CorridorMismatch` | `pi.corridor_id != corridor_id` |
| 9 | `MinTierMismatch` | `pi.min_tier != policy.min_tier` |
| 10 | `StaleProofTime` | `\|pi.now − ledger.timestamp\| > now_tolerance_secs` |
| 11 | `ProofInvalid` | the verifier returned `false` |
| 12 | `NullifierUsed` | replay — this nullifier already passed this corridor |
| 13 | `IssuerNotAccepted` | `pi.issuer_id ∉ policy.accepted_issuers` |
| 14 | `BadPublicInputs` | `public_inputs.len() != 9` or a malformed word |
| 15 | `RootEpochRegression` | `set_min_cred_epoch` given a value below the current floor |
| 17 | `NoPendingAdmin` | `accept_admin` without a prior `propose_admin` |
| 18 | `DisclosureMissing` | `pi.auditor_pubkey != policy.auditor_pubkey` |
| 19 | `CredEpochMismatch` | `pi.min_cred_epoch != policy.min_cred_epoch` |

`1–2` (`NotInitialized` / `AlreadyInitialized`), `7` (`RootMismatch`), and `16`
(`RelayerNotAllowed`) are retained for numbering stability but unused under
Option B.

---

## State rent

`corridor_attestation` stores one persistent entry per granted pass. Its TTL is
bumped (~2 years, clamped to the network max) on write **and on every
`is_cleared` / `pass_record` read**, so an actively-monitored pass never
expires. High-volume corridors still accrue rent; an archival design is tracked
in [#5](https://github.com/Sconce-Labs/corridor-contracts/issues/5). A nullifier
must not silently expire while the credential behind it could still be
presented.

---

## Security

Not audited — see [`SECURITY.md`](./SECURITY.md) for the contract-specific
threat model (mock verifier, issuer-key compromise, stale `min_cred_epoch`,
clock skew) and how to report a vulnerability privately. **Do not deploy to
mainnet with real value.**

---

## Contributing

Corridor participates in the **[Stellar Drips Wave](https://www.drips.network/wave/stellar)** —
issues labelled `drips` are reward-eligible. See
[`CONTRIBUTING.md`](./CONTRIBUTING.md) and the main repo's
[`DRIPS.md`](https://github.com/Sconce-Labs/corridor/blob/main/DRIPS.md).

- `cargo test --workspace`, `cargo fmt`, and `cargo clippy` must pass.
- New contract errors get a stable number and a doc comment.
- A public-input layout change means matching PRs on `corridor-circuits` and
  `corridor-sdk` plus an [`ABI.md`](./ABI.md) update.
- Conventional commits (`feat:`, `fix:`, `test:`, `docs:`, `chore:`).

## License

[Apache-2.0](./LICENSE) · see [`NOTICE`](./NOTICE).
