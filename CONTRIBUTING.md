# Contributing

`corridor-contracts` participates in the **Stellar Drips Wave** — see
[Sconce-Labs/corridor `DRIPS.md`](https://github.com/Sconce-Labs/corridor/blob/main/DRIPS.md).
Open issues are labelled `drips`.

## Setup

```bash
rustup target add wasm32v1-none
cargo test --workspace
cargo fmt --all -- --check
cargo build --release --target wasm32v1-none \
  -p corridor-registry -p corridor-attestation -p verifier-mock
```

On Windows GNU, `.cargo/config.toml` works around an `ld` export-table limit in
soroban-sdk's dependency tree.

## Rules

- One issue per PR; `Closes #N`.
- `cargo test --workspace` and `cargo fmt` must pass. New behaviour needs a test.
- **The public-input layout** (`crates/corridor_types/src/abi.rs`) is the source
  of truth mirrored in `ABI.md`, `corridor-circuits`, and `corridor-sdk`.
  Changing it is a coordinated PR across all four.
- **Poseidon2 conformance** (`crates/poseidon_conformance`) must stay green.
- Every new contract error gets a stable number and a doc-comment line.

## Layout

| Crate | Role |
|-------|------|
| `crates/corridor_types` | `errors` / `policy` / `abi` / `interfaces` modules |
| `crates/poseidon_conformance` | test-only Poseidon2 cross-impl check |
| `contracts/corridor_registry` | policy CRUD + `set_min_cred_epoch` + two-step admin |
| `contracts/corridor_attestation` | `enter` / `is_cleared` / nullifier ledger |
| `contracts/ultrahonk_verifier` | real UltraHonk verifier skeleton (M3) |
| `contracts/verifier_mock` | configurable verifier (tests + staging) |
