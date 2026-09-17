# ultrahonk_verifier

The real ZK verifier for Corridor.

Implements `corridor_types::Verifier`
(`verify(vk_hash, proof, public_inputs) -> bool`), so a corridor policy can
point its `verifier` at this contract instead of `verifier_mock` with no change
to `corridor_attestation`.

## Status

- ✅ VK stored at deploy time; `vk_hash()` exposed; `verify` enforces the VK pin.
- ✅ The UltraHonk verification engine (Fiat–Shamir transcript → sumcheck → Shplemini/KZG opening → BN254 pairing check) adapted from `indextree/ultrahonk_soroban_contract` and Protocol 25 BN254 host functions.
- ✅ Full test suite with real verification key and proof test artifacts.

Tracked in [#1](https://github.com/Sconce-Labs/corridor-contracts/issues/1) and
[#2](https://github.com/Sconce-Labs/corridor-contracts/issues/2).

## Deploy

```bash
stellar contract deploy --wasm target/wasm32v1-none/release/ultrahonk_verifier.wasm \
  --source corridor --network testnet -- --vk <path-or-hex-of-the-vk>
```

Then set that address + `vk_hash()` on a corridor's `CorridorPolicy`.
