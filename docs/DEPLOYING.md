# Deploying the Corridor contracts

## Prereqs

```bash
rustup target add wasm32v1-none
# stellar CLI >= 22
stellar keys generate corridor --network testnet --fund
```

## Build + deploy (testnet)

`scripts/deploy_testnet.sh` does all of this and writes `deployments/testnet.json`:

```bash
cargo build --release --target wasm32v1-none \
  -p corridor-registry -p corridor-attestation -p verifier-mock

OUT=target/wasm32v1-none/release
ADMIN=$(stellar keys address corridor)

VERIFIER=$(stellar contract deploy --wasm $OUT/verifier_mock.wasm \
  --source corridor --network testnet)

REGISTRY=$(stellar contract deploy --wasm $OUT/corridor_registry.wasm \
  --source corridor --network testnet -- --admin $ADMIN)

ATTESTATION=$(stellar contract deploy --wasm $OUT/corridor_attestation.wasm \
  --source corridor --network testnet -- --registry $REGISTRY)
```

## Register a corridor + go live

```bash
# 1. operator registers a policy (roots start empty)
stellar contract invoke --id $REGISTRY --source corridor --network testnet --send=yes -- \
  register --corridor_id <cid> --policy '<json>'

# 2. a relayer syncs the Midnight roots (see Sconce-Labs/corridor-relayer)
stellar contract invoke --id $REGISTRY --source corridor --network testnet --send=yes -- \
  post_root --relayer <addr> --corridor_id <cid> \
  --credential_root <root> --revocation_root <root> --epoch 1

# 3. holders call enter() on $ATTESTATION with a proof (Sconce-Labs/corridor-sdk)
# 4. the operator's payout contract calls is_cleared() before paying out
```

## Swapping the verifier (M3)

Deploy `ultrahonk_verifier` with the real VK, then `update_policy` the corridor
to point `verifier` at it and set `vk_hash` to `ultrahonk_verifier.vk_hash()`.
No change to `corridor_attestation`.

## Mainnet

Not yet. Blocked on: the real verifier (M3), the Midnight registry on Preprod
(corridor#M4), an external audit, and relayer decentralisation (corridor-relayer#2).
