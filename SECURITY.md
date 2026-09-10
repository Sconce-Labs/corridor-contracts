# Security

Open a private security advisory or email the maintainers. Not a public issue.

## Threat model (contract-specific)

| Risk | Impact | Status / mitigation |
|------|--------|---------------------|
| `verifier_mock` deployed to a real corridor | Any proof accepted | It is clearly named; a policy's `verifier` is the operator's choice. M3 ships the real UltraHonk verifier. |
| Verifier soundness bug | Fake passes | Use the audited reference verifier; pin `vk_hash`; every policy has a `paused` kill switch. |
| Compromised issuer key | Bad statements signed until noticed | Short `expiry` caps the window; the issuer bumps `cred_epoch` and operators raise `min_cred_epoch` (`set_min_cred_epoch`, monotonic) to bulk-revoke; a corridor can drop the issuer from `accepted_issuers` instantly. |
| Stale `min_cred_epoch` on a policy | A revoked cohort keeps passing | Operator must mirror each accepted issuer's epoch; `MinCredEpochSet` event + monotonic guard. |
| Nullifier state expiry | A pass entry could expire while its credential could still be presented → replay | TTL ~2y, bumped on write and on `is_cleared`/`pass_record` reads; **archival design owed before high volume** ([#5](https://github.com/Sconce-Labs/corridor-contracts/issues/5)). |
| `now` in the proof far from real time | Just-expired credential slips through within tolerance | `now_tolerance_secs` window enforced in `enter` (documented, acceptable). |
| Poseidon2 / Schnorr divergence | Proofs silently fail to verify | `crates/poseidon_conformance` pins the vector; the SDK signer is pinned to `noir-lang/schnorr` and proven against the circuit via `nargo execute` in CI. |

## Not audited

No external audit yet. Do not deploy to mainnet with real value.
