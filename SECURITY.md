# Security

Open a private security advisory or email the maintainers. Not a public issue.

## Threat model (contract-specific)

| Risk | Impact | Status / mitigation |
|------|--------|---------------------|
| `verifier_mock` deployed to a real corridor | Any proof accepted | It is clearly named; a policy's `verifier` is the operator's choice. M3 ships the real UltraHonk verifier. |
| Verifier soundness bug | Fake passes | Use the audited reference verifier; pin `vk_hash`; every policy has a `paused` kill switch. |
| Dishonest relayer posts a bad root via `post_root` | Invalid credentials admitted / valid ones censored | `epoch` is strictly monotonic on-chain; `post_root` emits a `ROOT` event tagged with the relayer. Multi-relayer quorum is the fix (corridor-relayer#2). |
| Nullifier state expiry | A pass entry could expire while its credential is still valid → replay | TTL is ~30 days today; **archival design owed before high volume** ([#5](https://github.com/Sconce-Labs/corridor-contracts/issues/5)). |
| `now` in the proof far from real time | Expired credential slips through | `now_tolerance_secs` window enforced in `enter`. |
| Poseidon2 divergence | Merkle roots silently disagree across chains | `crates/poseidon_conformance` pins the vector. |

## Not audited

No external audit yet. Do not deploy to mainnet with real value.
