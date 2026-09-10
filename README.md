# corridor-contracts

[![CI](https://github.com/Sconce-Labs/corridor-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Sconce-Labs/corridor-contracts/actions/workflows/ci.yml)

Soroban smart contracts for **Corridor** — a portable proof of eligibility for
cross-border payments. This repo is the Stellar side: corridor policy, on-chain
ZK proof verification, the nullifier ledger, and payout gating.

- Product + full architecture: **[Sconce-Labs/corridor](https://github.com/Sconce-Labs/corridor)**
- Noir circuit: **[Sconce-Labs/corridor-circuits](https://github.com/Sconce-Labs/corridor-circuits)**
- Client SDK: **[Sconce-Labs/corridor-sdk](https://github.com/Sconce-Labs/corridor-sdk)**
- Public-input ABI (source of truth): [`ABI.md`](./ABI.md)

## Workspace

| Crate | Kind | Role |
|-------|------|------|
| `crates/corridor_types` | rlib | Shared types, errors, the `PI_*` public-input ABI, `Verifier` / `Registry` cross-contract interfaces |
| `crates/poseidon_conformance` | test-only | Asserts Soroban's Poseidon2 matches the circuit + SDK vector ([`ABI.md`](./ABI.md) "Hash conformance") |
| `contracts/corridor_registry` | contract | Per-corridor `CorridorPolicy` (accepted issuers, min tier, `min_cred_epoch` revocation floor); two-step admin |
| `contracts/corridor_attestation` | contract | `enter(corridor_id, proof, public_inputs)` → bind to policy → verify → burn nullifier → record `PassRecord`. `is_cleared()` for payout gating |
| `contracts/ultrahonk_verifier` | contract | Real UltraHonk verifier skeleton (M3) behind the `Verifier` interface |
| `contracts/verifier_mock` | contract | Configurable pass/fail verifier for tests and staged rollout. **Not for production** — the real UltraHonk verifier replaces it. |

## Build & test

```bash
cargo test --workspace          # host tests, no network
cargo build --release --target wasm32v1-none \
  -p corridor-registry -p corridor-attestation -p verifier-mock
```

Requires the `wasm32v1-none` target (`rustup target add wasm32v1-none`) and the
`stellar` CLI ≥ 22. On Windows GNU, `.cargo/config.toml` works around an `ld`
export-table limit hit by soroban-sdk's dependency tree.

## Testnet deployment

> **Stale — pending redeploy.** The addresses in
> [`deployments/testnet.json`](./deployments/testnet.json) ran the pre-Option-B
> ABI (10 public inputs, synced roots, `post_root`). The current contracts use
> the 9-input Option B ABI. A redeploy + refreshed record is
> [corridor ROADMAP](https://github.com/Sconce-Labs/corridor/blob/main/ROADMAP.md)
> **M2**.

The earlier run verified `register` → `post_root` → `enter` → `is_cleared ==
true` with replay rejected; `post_root` no longer exists.

Deploy your own: `scripts/deploy_testnet.sh`, then `scripts/demo.sh` (both
being updated for Option B alongside the M2 redeploy).

## Integration for a corridor operator

1. `corridor_registry.register(corridor_id, policy)` with your operator key —
   set `accepted_issuers` (the `Poseidon2(pk.x, pk.y)` ids), `min_tier`,
   `min_cred_epoch`, `auditor_pubkey`, `verifier`, `vk_hash`.
2. When an accepted issuer bulk-revokes (bumps its epoch on Midnight), raise
   your floor: `corridor_registry.set_min_cred_epoch(corridor_id, epoch)`
   (monotonic).
3. In your payout contract, before releasing funds:
   `corridor_attestation.is_cleared(corridor_id, nullifier)`.

That's the whole integration surface.

## Events

| Contract | Event | Topics | Data |
|----------|-------|--------|------|
| registry | `Registered` | `corridor_id`, `operator` | `min_tier` |
| registry | `PolicyUpdated` | `corridor_id` | `min_tier`, `required_disclosures`, `min_cred_epoch`, `verifier`, `vk_hash`, `paused` |
| registry | `PausedSet` | `corridor_id` | `paused` |
| registry | `MinCredEpochSet` | `corridor_id` | `min_cred_epoch` |
| registry | `AdminTransferred` | `old`, `new` | — |
| attestation | `PassGranted` | `corridor_id`, `nullifier` | `tag`, `passes` |

Defined with `#[contractevent]` in each contract's `events.rs`.

## State-rent note

`corridor_attestation` stores one persistent entry per granted pass, TTL bumped
(~2y, clamped to the network max) on write and on every `is_cleared` /
`pass_record` read — so an actively-monitored pass never expires. High-volume
corridors still accrue rent; an archival path is on the roadmap
([#5](https://github.com/Sconce-Labs/corridor-contracts/issues/5)). Nullifiers
must not silently expire while a credential could still be presented.

## Contributing

Corridor participates in the **Stellar Drips Wave**. Issues labelled `drips` are
reward-eligible — see the [main repo](https://github.com/Sconce-Labs/corridor)
`DRIPS.md`. `cargo test --workspace` and `cargo fmt` must pass. If you change the
public-input layout, update [`ABI.md`](./ABI.md) and open matching PRs on
`corridor-circuits` and `corridor-sdk`.

## License

Apache-2.0
