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
| `contracts/corridor_registry` | contract | Per-corridor `CorridorPolicy`; a relayer posts Midnight-synced roots via `post_root` |
| `contracts/corridor_attestation` | contract | `enter(corridor_id, proof, public_inputs)` → bind to policy → verify → burn nullifier → record `PassRecord`. `is_cleared()` for payout gating |
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

Live on Stellar testnet — see [`deployments/testnet.json`](./deployments/testnet.json).

| Contract | Address |
|----------|---------|
| `corridor_registry` | `CB6LZV6TJN6YZ2O7FVLNRCJMRVBXCDG6JFFREHGY2BD5K4EYWJ6WKT2K` |
| `corridor_attestation` | `CCAGXABIZWHNLA754LSQCFPA35VLJZEH24MD5OGJNIEMFQHZ7LWQD5AR` |
| `verifier_mock` | `CDT4ZVOIAI5JN4TC3WZYIBJ3NOJENZWKD2ZNOTSVVZBZBZ5GMOSNJEQP` |

`register` → `post_root` → `enter` → `is_cleared == true` verified end-to-end;
replay rejected with `NullifierUsed`.

Deploy your own: `scripts/deploy_testnet.sh`, then `scripts/demo.sh`.

## Integration for a corridor operator

1. `corridor_registry.register(corridor_id, policy)` with your operator key.
2. Keep `credential_root` / `revocation_root` fresh via `post_root` (run the
   relayer, or do it yourself while it's being built).
3. In your payout contract, before releasing funds:
   `corridor_attestation.is_cleared(corridor_id, nullifier)`.

That's the whole integration surface.

## State-rent note

`corridor_attestation` stores one persistent entry per granted pass, TTL-extended
to ~30 days. High-volume corridors accrue rent; an archival path is on the
roadmap. Nullifiers must not silently expire while a credential is still valid.

## Contributing

Corridor participates in the **Stellar Drips Wave**. Issues labelled `drips` are
reward-eligible — see the [main repo](https://github.com/Sconce-Labs/corridor)
`DRIPS.md`. `cargo test --workspace` and `cargo fmt` must pass. If you change the
public-input layout, update [`ABI.md`](./ABI.md) and open matching PRs on
`corridor-circuits` and `corridor-sdk`.

## License

Apache-2.0
