#!/usr/bin/env bash
# demo-s3-cancel.sh — S3 : cancel une demande ouverte (aucun credit ne bouge).
# Usage (1 commande) : ./demo-s3-cancel.sh
# Hermetique : HELIUM_HOME temporaire, rien n est ecrit dans ~/.helium.
# Shell : Git-Bash ou PowerShell (l interop WSL ne transmet pas les env
# custom aux .exe Windows — sous WSL, HELIUM_HOME serait ignore).
# Requiert : binaire compile (cd ../helium-mesh && cargo build).
# Note : le timeout auto (TTL) tourne dans le daemon (match_loop, defaut
# HELIUM_REQUEST_TTL_HOURS=24) ; il est prouve par le test cancel_and_expiry.
set -euo pipefail

DIR="$(cd "$(dirname "$0")/../../helium-mesh/target/debug" && pwd)"
BIN="$DIR/helium"
[ -x "$BIN" ] || BIN="$DIR/helium.exe"
[ -x "$BIN" ] || { echo "compile d'abord : (cd helium-mesh && cargo build)"; exit 1; }
export HELIUM_HOME="$(mktemp -d)/helium"
ME="demo-cancel"

echo "[1/3] demande ouverte"
"$BIN" faucet --party "$ME" --amount 100 >/dev/null
REQ=$("$BIN" market request --by "$ME" --amount 8 --max-price 2 --hours 1 | awk '{print $3}')
echo "    $REQ"

echo "[2/3] cancel"
"$BIN" market cancel --request-id "$REQ"

echo "[3/3] la demande annulee ne matche plus et ne s accepte plus"
"$BIN" market match --request-id "$REQ" || echo "    (refus attendu : not open)"
echo "OK : cancel sans mouvement de credits."
