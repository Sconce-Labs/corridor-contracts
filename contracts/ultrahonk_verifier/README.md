# ultrahonk_verifier

The real ZK verifier for Corridor — **wired (M3 step 1)**.

Implements the `Verifier` wire ABI (`verify(vk_hash, proof, public_inputs) -> bool`),
so a corridor policy can point its `verifier` at this contract instead of
`verifier_mock` with no change to `corridor_attestation`.

## What it does

- VK (1760 bytes, Barretenberg `UltraKeccakFlavor` layout) is set once at
  deployment and **validated by parsing before storage** — a malformed VK
  aborts the deploy. There is no admin key or upgrade path.
- `verify` is **fail-closed**: wrong `vk_hash` pin, wrong input count
  (9 words, the corridor `PI_*` layout), wrong proof length
  (14 592 bytes), or any failed check → `false`. It never traps on
  caller-supplied data.
- The verification itself (Fiat–Shamir transcript → sumcheck → Shplemini →
  BN254 pairing check) runs in `crates/ultrahonk_core`, vendored
  byte-identical from [NethermindEth/ultrahonk-rust-verifier] at
  `097da17df8b0b971d71885566034958a8caebab4` — the commit OpenZeppelin
  audited (Aug 2026): 0 Critical / 0 High / 0 Medium, 5 Low (remediated).
  The core uses only native Soroban host crypto (`bn254` field ops,
  `g1_msm`, `pairing_check`, `keccak256`).

## Toolchain pin (important)

Proofs and VKs must be produced by **Barretenberg v0.87.0**
(`UltraKeccakFlavor`). A bb version mismatch is *not* caught by the length
checks — it fails silently at the transcript/pairing stage (real proofs
simply do not verify). Prove the corridor circuit with bb 0.87.0, or
coordinate a core bump; see
`crates/ultrahonk_core/VERIFIER_PROVENANCE.md`.

## SDK note

This contract builds on **soroban-sdk 28** (the core needs SDK-28 host APIs:
`Bn254Fr`, `g1_msm`, `g1_is_on_curve`) while the rest of the workspace stays
on SDK 25. The graphs are deliberately isolated — this crate does not depend
on `corridor_types`; the wire ABI is SDK-agnostic. Cross-compilation must go
through `stellar contract build` (SDK 28 refuses a build system that cannot
do spec shaking v2):

```bash
stellar contract build --package ultrahonk-verifier
# → target/wasm32v1-none/release/ultrahonk_verifier.wasm
```

## Deploy

```bash
stellar contract deploy --wasm target/wasm32v1-none/release/ultrahonk_verifier.wasm \
  --source corridor --network testnet -- --vk <path-or-hex-of-the-vk>
```

Then set that address + `vk_hash()` on the corridor's `CorridorPolicy`.
Anyone can deploy an instance with an arbitrary VK, so **pin `vk_hash` of the
known-good VK** in the policy — that pin is what makes the trust root the
circuit, not the deployer. Independent check: `vk_bytes()` returns the stored
VK for audit.

## Status

- ✅ Full UltraHonk verification wired over the audited core (step 1).
- ✅ Adapter unit tests (VK pin, length guards, constructor validation).
- ✅ End-to-end proof tests with real `corridor_eligibility` artifacts (step
  2): the committed proof (noir 1.0.0-beta.9 + bb 0.87.0) verifies through
  the full pipeline in the Soroban host, and mutated / truncated proofs and
  reordered public inputs are rejected. The core's transcript-determinism
  test also runs unskipped (its `simple_circuit` fixture is committed; the
  regenerated VK is byte-identical to upstream's pinned artifact hash).
- ✅ **Live on testnet** (step 3): deployed with the real VK and wired into a
  corridor policy — a fresh corridor proof granted a pass on-chain
  (`PassGranted`, `is_cleared == true`), a replay was rejected with
  `NullifierUsed` (#12) and a tampered proof with `ProofInvalid` (#11).
  Addresses + tx hashes: `deployments/testnet.json` (`m3RealVerifier`).

[NethermindEth/ultrahonk-rust-verifier]: https://github.com/NethermindEth/ultrahonk-rust-verifier
