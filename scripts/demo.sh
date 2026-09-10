#!/usr/bin/env bash
# End-to-end demo against the mock verifier (Option B): register a corridor,
# present a proof, confirm the pass, confirm replay is rejected.
#   VERIFIER=... REGISTRY=... ATTESTATION=... ./scripts/demo.sh
#
# The mock verifier returns true, so `proof` / the witness are not checked here
# — this exercises the policy binding and the nullifier ledger. A real proof
# from corridor-sdk's buildWitness is used once the UltraHonk verifier is wired
# (M3).
set -euo pipefail

NET=testnet
SRC=${CORRIDOR_SOURCE:-corridor}
: "${VERIFIER:?set VERIFIER}" "${REGISTRY:?set REGISTRY}" "${ATTESTATION:?set ATTESTATION}"

OP=$(stellar keys address "$SRC")

# 32-byte hex helpers
CID=0000000000000000000000000000000000000000000000000000000000000004
ISSUER=0000000000000000000000000000000000000000000000000000000000000007
NULLIFIER=3333333333333333333333333333333333333333333333333333333333333333
VK=0909090909090909090909090909090909090909090909090909090909090909
ZERO=0000000000000000000000000000000000000000000000000000000000000000

MIN_TIER=2
MIN_CRED_EPOCH=1

inv() { stellar contract invoke --id "$1" --source "$SRC" --network "$NET" -- "${@:2}"; }

echo "── register corridor ────────────────────────────────────────"
inv "$REGISTRY" register --corridor_id "$CID" --policy "{
  \"operator\":\"$OP\",
  \"accepted_issuers\":[\"$ISSUER\"],
  \"min_tier\":$MIN_TIER,
  \"required_disclosures\":0,
  \"min_cred_epoch\":$MIN_CRED_EPOCH,
  \"verifier\":\"$VERIFIER\",
  \"vk_hash\":\"$VK\",
  \"auditor_pubkey\":\"$ZERO\",
  \"now_tolerance_secs\":300,
  \"paused\":false
}"

echo "── build public inputs (9, Option B; now = ledger time) ─────"
# order: corridor_id, min_tier, now, nullifier, disclosed_tag,
#        issuer_id, min_cred_epoch, auditor_pubkey, auditor_blob
NOW=$(printf '%064x' "$(date +%s)")
TIER=$(printf '%064x' "$MIN_TIER")
TAG1=$(printf '%064x' 1)
EPOCH=$(printf '%064x' "$MIN_CRED_EPOCH")
PUB="[\"$CID\",\"$TIER\",\"$NOW\",\"$NULLIFIER\",\"$TAG1\",\"$ISSUER\",\"$EPOCH\",\"$ZERO\",\"$ZERO\"]"

echo "── enter the corridor ──────────────────────────────────────"
inv "$ATTESTATION" enter --corridor_id "$CID" --proof aabbccdd --public_inputs "$PUB"

echo "── is_cleared? (expect true) ───────────────────────────────"
inv "$ATTESTATION" is_cleared --corridor_id "$CID" --nullifier "$NULLIFIER"

echo "── replay (expect NullifierUsed / error) ───────────────────"
if inv "$ATTESTATION" enter --corridor_id "$CID" --proof aabbccdd --public_inputs "$PUB"; then
  echo "!! replay unexpectedly succeeded"; exit 1
else
  echo "ok — replay rejected"
fi
