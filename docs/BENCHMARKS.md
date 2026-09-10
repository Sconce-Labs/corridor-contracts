# Benchmarks

## Wasm size (`--release --target wasm32v1-none`)

| Contract | Approx. size |
|----------|-------------:|
| `corridor_attestation` | ~22 KB |
| `corridor_registry` | ~14 KB |
| `verifier_mock` | ~3.5 KB |
| `ultrahonk_verifier` (skeleton) | ~5 KB |

## `enter()` resource budget

Captured against the `verifier_mock` on testnet with `scripts/demo.sh`. The
real cost is dominated by the verifier call — the UltraHonk verifier (M3) will
be the large line item. Re-run and record here once it lands:

```bash
stellar contract invoke --id $ATTESTATION --source corridor --network testnet \
  --cost -- enter --corridor_id $CID --proof aa --public_inputs "$PUB"
```

| Path | CPU insns | Mem bytes | Read entries | Write entries |
|------|----------:|----------:|-------------:|--------------:|
| `enter` (mock verifier) | _tbd — capture_ | | | |
| `is_cleared` | _tbd_ | | | |

Tracked in [#3](https://github.com/Sconce-Labs/corridor-contracts/issues/3).

## Decision gate

If `enter` with the real UltraHonk verifier exceeds a Soroban transaction's
resource limits, or costs more than a payment can bear, fall back to a Groth16
verifier via the BN254 `pairing_check` host function (feature parity with
Ethereum's EIP-197). The `Verifier` interface makes that swap contained.
