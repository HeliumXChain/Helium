#!/usr/bin/env bash
# helium-health.sh — controle sante provider (tunnel, daemon, VM, guest, disque).
# Usage: sudo ./helium-health.sh [--json]
# Sortie: lignes OK/WARN/FAIL + code retour (0 ok, 1 echec, 2 avertissements).
set -uo pipefail

JSON=0
[ "${1:-}" = "--json" ] && JSON=1
fails=0; warns=0
results=""

check() { # check <nom> <commande...>
  local name="$1"; shift
  if out="$("$@" 2>&1)"; then
    results="${results}OK $name\n"
  else
    results="${results}FAIL $name :: $(echo "$out" | head -1)\n"
    fails=$((fails + 1))
  fi
}

check_warn() { # check_warn <nom> <commande...>
  local name="$1"; shift
  if out="$("$@" 2>&1)"; then
    results="${results}OK $name\n"
  else
    results="${results}WARN $name :: $(echo "$out" | head -1)\n"
    warns=$((warns + 1))
  fi
}

SUDO=""
[ "$(id -u)" != "0" ] && SUDO="sudo -n "

# 1. KVM + binaire
check "kvm-present" test -e /dev/kvm
check "helium-bin" command -v helium

# 2. Tunnel WireGuard
check "wg-iface" ip link show helium-poc
if $SUDO wg show helium-poc latest-handshakes 2>/dev/null | grep -q .; then
  results="${results}OK wg-handshake\n"
else
  # handshake absent = peut-etre normal (pas de trafic recent)
  results="${results}WARN wg-handshake :: aucun handshake (tunnel idle ?)\n"
  warns=$((warns + 1))
fi

# 3. Services systemd
check "svc-wg" systemctl is-active --quiet wg-quick@helium-poc
check "svc-daemon" systemctl is-active --quiet helium-daemon
check_warn "svc-vm" systemctl is-active --quiet helium-vm

# 4. MicroVM + guest
if [ -S /tmp/fc-helium.socket ]; then
  results="${results}OK vm-socket\n"
  GKEY=/srv/helium-vm/ubuntu-22.04.id_rsa
  [ -f /srv/helium-vm/vm_key ] && GKEY=/srv/helium-vm/vm_key
  if ssh -i "$GKEY" -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
      -o ConnectTimeout=8 root@172.16.0.2 true 2>/dev/null; then
    results="${results}OK guest-ssh\n"
  else
    results="${results}FAIL guest-ssh :: injoignable\n"
    fails=$((fails + 1))
  fi
else
  results="${results}WARN vm-socket :: VM eteinte (normal si pas de match)\n"
  warns=$((warns + 1))
fi

# 5. Disque + DB
root_free=$(df -m / | awk 'NR==2{print $4}')
if [ "${root_free:-0}" -lt 1024 ]; then
  results="${results}FAIL disk :: ${root_free}Mo libres\n"
  fails=$((fails + 1))
else
  results="${results}OK disk :: ${root_free}Mo libres\n"
fi
user_home=$(getent passwd "${SUDO_USER:-$(id -un)}" | cut -d: -f6)
[ -f "$user_home/.helium/market.db" ] && results="${results}OK market-db\n" || {
  results="${results}WARN market-db :: absente\n"
  warns=$((warns + 1))
}

if [ "$JSON" = "1" ]; then
  printf '{"fails":%d,"warns":%d}\n' "$fails" "$warns"
else
  printf "$results"
fi
[ "$fails" -eq 0 ] || exit 1
[ "$warns" -eq 0 ] || exit 2
exit 0
