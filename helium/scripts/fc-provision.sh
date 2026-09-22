#!/usr/bin/env bash
# fc-provision.sh — rend une microVM Helium utilisable pour les workloads.
# Idempotent : chaque etape verifie avant d'agir. A lancer sur le provider.
# Usage: sudo ./fc-provision.sh [--guest-ip 172.16.0.2] [--key DIR/ubuntu-22.04.id_rsa] [--dir /srv/helium-vm] [--python]
#   --python : installe aussi pip + numpy (lourd, ~60Mo, pour l'entrainement ML)
set -euo pipefail

GUEST_IP="172.16.0.2"
KEY=""
DIR="/srv/helium-vm"
WITH_PYTHON=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --guest-ip) GUEST_IP="$2"; shift 2 ;;
    --key) KEY="$2"; shift 2 ;;
    --dir) DIR="$2"; shift 2 ;;
    --python) WITH_PYTHON=1; shift ;;
    *) echo "usage: $0 [--guest-ip IP] [--key PATH] [--dir DIR] [--python]"; exit 1 ;;
  esac
done
[ -z "$KEY" ] && KEY="$DIR/ubuntu-22.04.id_rsa"

if [ "$(id -u)" != "0" ]; then exec sudo -E "$0" "$@"; fi

G="ssh -i $KEY -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=10 root@$GUEST_IP"
ok()   { echo "[fc-provision][OK] $*"; }
step() { echo "[fc-provision] $*"; }

step "attente SSH $GUEST_IP…"
for _ in $(seq 1 24); do $G true 2>/dev/null && break; sleep 5; done
$G true || { echo "[fc-provision][ERR] guest injoignable" >&2; exit 1; }
ok "SSH ok"

# 1. route par defaut (NAT via tap)
if $G 'ip route show default' 2>/dev/null | grep -q .; then
  ok "route par defaut presente"
else
  step "ajout route par defaut…"
  $G 'ip route replace default via 172.16.0.1 dev eth0'
  ok "route par defaut ok"
fi

# 2. DNS
if $G 'getent hosts pypi.org' >/dev/null 2>&1; then
  ok "DNS ok"
else
  step "DNS…"
  $G 'printf "nameserver 8.8.8.8\nnameserver 1.1.1.1\n" > /etc/resolv.conf'
  $G 'getent hosts pypi.org' >/dev/null || { echo "[fc-provision][ERR] DNS ko" >&2; exit 1; }
  ok "DNS ok"
fi

# 3. CA (sans quoi HTTPS echoue dans l'image CI strippee)
if $G 'ls /etc/ssl/certs/ca-certificates.crt' >/dev/null 2>&1; then
  ok "CA presentes"
else
  step "CA depuis le host…"
  cp /etc/ssl/certs/ca-certificates.crt /tmp/helium-ca.crt
  scp -i "${KEY}" -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
    /tmp/helium-ca.crt "root@${GUEST_IP}:/tmp/ca.crt" >/dev/null
  rm -f /tmp/helium-ca.crt
  $G 'mkdir -p /etc/ssl/certs && cp /tmp/ca.crt /etc/ssl/certs/ca-certificates.crt && rm -f /tmp/ca.crt'
  $G 'curl -s -o /dev/null -w "%{http_code}" https://pypi.org/simple/ | grep -q 200' \
    || { echo "[fc-provision][ERR] HTTPS ko" >&2; exit 1; }
  ok "CA ok (HTTPS verifie)"
fi

# 4. Python ML (optionnel)
if [ "$WITH_PYTHON" = "1" ]; then
  if $G 'python3 -c "import numpy"' >/dev/null 2>&1; then
    ok "numpy present"
  else
    step "pip + numpy…"
    $G 'curl -fsSL https://bootstrap.pypa.io/get-pip.py -o /tmp/get-pip.py && python3 /tmp/get-pip.py 2>&1 | tail -1'
    $G 'python3 -m pip install -q --break-system-packages numpy 2>&1 | tail -1'
    $G 'python3 -c "import numpy; print(numpy.__version__)"' || { echo "[fc-provision][ERR] numpy ko" >&2; exit 1; }
    ok "numpy ok"
  fi
fi

ok "guest pret : $GUEST_IP"
