# Contributing to corridor-contracts

Thanks for helping build Corridor — a portable proof-of-eligibility for
cross-border payments (Stellar/Soroban + Noir/UltraHonk). Newcomers welcome:
look for issues labeled `good first issue`. The project overview lives in the
[hub repo](https://github.com/Sconce-Labs/corridor).

## Ground rules

- **One issue per PR.** Reference it with `Closes #NNN`.
- Conventional commits (`feat:`, `fix:`, `test:`, `docs:`, `chore:`).
- Don't weaken a trust assumption without updating
  `ARCHITECTURE.md §6` in the hub repo.
- Apache-2.0; by contributing you agree your work is licensed under it.
- We follow the [Code of Conduct](./CODE_OF_CONDUCT.md).

## Setup

```bash
rustup target add wasm32v1-none
cargo test --workspace --locked
```

## What you can work on

- Soroban contracts (Rust, `soroban-sdk`): `contracts/corridor_registry`,
  `contracts/corridor_attestation`, `contracts/verifier_mock`, and the real
  `contracts/ultrahonk_verifier` (vendored, OpenZeppelin-audited core).
- Circuit fixtures + verifier tests: `tests/circuits/` (artifact dirs are
  re-included in git — commit only `proof` / `vk` / `public_inputs`).
- CI, docs, gas benchmarks, and the audit-hardening backlog.

## Gates (all must pass)

```bash
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Building the verifier wasm additionally needs the `stellar` CLI ≥ 25.2:
`stellar contract build --package ultrahonk-verifier` (soroban-sdk 28 refuses
older build systems).

## Review & merging

Maintainers aim to review within 48h during active contribution waves. Small
PRs get reviewed first — keep diffs reviewable. CI must be green before merge;
if CI fails for reasons outside your control, say so in the PR and we will
pick it up.
