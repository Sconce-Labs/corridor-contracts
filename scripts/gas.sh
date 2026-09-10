#!/usr/bin/env bash
# Capture the resource budget of the core calls and print a Markdown row.
#   VERIFIER=... REGISTRY=... ATTESTATION=... ./scripts/gas.sh
set -euo pipefail
NET=testnet
SRC=${CORRIDOR_SOURCE:-corridor}
: "${ATTESTATION:?}" "${REGISTRY:?}"

CID=0000000000000000000000000000000000000000000000000000000000000004
NULL=3333333333333333333333333333333333333333333333333333333333333333

echo "| call | output |"
echo "|------|--------|"
for call in \
  "is_cleared --id $ATTESTATION -- is_cleared --corridor_id $CID --nullifier $NULL" \
  "passes     --id $ATTESTATION -- passes --corridor_id $CID" \
  "get_policy --id $REGISTRY -- get_policy --corridor_id $CID"
do
  name=${call%% *}
  args=${call#* }
  out=$(stellar contract invoke --source "$SRC" --network "$NET" --cost $args 2>&1 | grep -iE "cpu|mem|instructions|entries" | tr '\n' ' ' || true)
  echo "| \`$name\` | $out |"
done

echo
echo "Paste the rows into docs/BENCHMARKS.md."
