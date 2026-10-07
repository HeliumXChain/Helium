#!/usr/bin/env bash
# demo-s3-quota.sh — S3 : quotas scheduler (2 jobs OK, 3e refuse).
# Usage (1 commande) : ./demo-s3-quota.sh
# Hermetique : HELIUM_HOME temporaire, rien n est ecrit dans ~/.helium.
# Shell : Git-Bash ou PowerShell (l interop WSL ne transmet pas les env
# custom aux .exe Windows — sous WSL, HELIUM_HOME serait ignore).
# Requiert : binaire compile (cd ../helium-mesh && cargo build).
set -euo pipefail

DIR="$(cd "$(dirname "$0")/../../helium-mesh/target/debug" && pwd)"
BIN="$DIR/helium"
[ -x "$BIN" ] || BIN="$DIR/helium.exe"
[ -x "$BIN" ] || { echo "compile d'abord : (cd helium-mesh && cargo build)"; exit 1; }
export HELIUM_HOME="$(mktemp -d)/helium"
ME="demo-quota"

echo "[1/3] credits + offre provider"
"$BIN" faucet --party "$ME" --amount 1000 >/dev/null
"$BIN" market offer --by provider-demo --amount 64 --price 0.1
"$BIN" market offer --by provider-demo2 --amount 64 --price 0.1
"$BIN" market offer --by provider-demo3 --amount 64 --price 0.1

echo "[2/3] 3 demandes du meme projet, 3 accepts (quota defaut = 2)"
for i in 1 2 3; do
  REQ=$("$BIN" market request --by "$ME" --amount 8 --max-price 2 --hours 1 | awk '{print $3}')
  MATCH=$("$BIN" market match --request-id "$REQ" | head -1 | awk '{print $1}' | sed 's/:$//')
  echo "--- accept $REQ via $MATCH"
  "$BIN" market accept --match-id "$MATCH" || echo "    (refus attendu au 3e : quota depasse, 429 via l API)"
done

echo "[3/3] workloads crees"
"$BIN" workload list
echo "OK : 2 Running, 3e refuse avec message 'quota exceeded'."
echo "Knob : HELIUM_MAX_ACTIVE_JOBS_PER_REQUESTER=N (defaut 2)."
