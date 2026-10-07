#!/usr/bin/env bash
# demo-live.sh — vitrine Q5 + bench en 3 etapes (hermetique).
set -euo pipefail
export HELIUM_HOME="$(mktemp -d)/helium"
BIN="./helium-mesh/target/debug/helium.exe"
echo "=== 1. templates disponibles"
"$BIN" workload templates 2>/dev/null
echo "=== 2. bench rapide (10s)"
"$BIN" bench --seconds 10 2>/dev/null
echo "=== 3. job via template"
"$BIN" workload run --template batch-scan --name demo-live 2>/dev/null
"$BIN" workload list 2>/dev/null
echo "DEMO-OK"
