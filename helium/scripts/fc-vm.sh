#!/usr/bin/env bash
# fc-vm.sh — boot/stop/ssh/expose une microVM Firecracker (provider generique)
# Usage:
#   sudo ./fc-vm.sh up [--vcpu 2] [--mem 2048] [--guest-ip 172.16.0.2] [--tap tap0] [--dir /srv/helium-vm]
#   sudo ./fc-vm.sh ssh -- <commande dans le guest>
#   sudo ./fc-vm.sh expose <guest-port> [host-port] [--wg-if helium-poc]
#   sudo ./fc-vm.sh down
# Zero chemin en dur perso : tout est parametre, artefacts CI telecharges si absents.
set -euo pipefail

if [ "$(id -u)" != "0" ]; then exec sudo -E "$0" "$@"; fi

CMD="${1:-help}"; [ "$#" -gt 0 ] && shift || true
VCPU=2; MEM=2048; GUEST_IP=172.16.0.2; TAP=tap0; DIR=/srv/helium-vm
WG_IF=helium-poc; FC_VER=1.5.1
while [ "$#" -gt 0 ]; do
  case "$1" in
    --vcpu) VCPU="$2"; shift 2 ;;
    --mem) MEM="$2"; shift 2 ;;
    --guest-ip) GUEST_IP="$2"; shift 2 ;;
    --tap) TAP="$2"; shift 2 ;;
    --dir) DIR="$2"; shift 2 ;;
    --wg-if) WG_IF="$2"; shift 2 ;;
    --) shift; break ;;
    *) break ;;
  esac
done

TAP_IP="${GUEST_IP%.*}.1"
SOCK=/tmp/fc-helium.socket
# Cle unique par VM si elle existe, sinon cle CI (1er boot seulement).
# La cle CI est PUBLIQUE (artefact Firecracker) : rotate_guest_key() la
# remplace des le 1er boot. Le fichier CI est garde pour re-bootstrap.
VM_KEY="$DIR/vm_key"
KEY="$DIR/ubuntu-22.04.id_rsa"
[ -f "$VM_KEY" ] && KEY="$VM_KEY"
SSH="ssh -i $KEY -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=10 root@$GUEST_IP"

info() { echo "[fc-vm] $*"; }
warn() { echo "[fc-vm][WARN] $*" >&2; }
need() { command -v "$1" >/dev/null 2>&1 || { echo "[fc-vm][ERR] missing: $1" >&2; return 1; }; }

ensure_artifacts() {
  mkdir -p "$DIR"
  base="https://s3.amazonaws.com/spec.ccfc.min/firecracker-ci/v1.5/x86_64"
  [ -f "$DIR/vmlinux-5.10.186" ] || curl -fsSL "$base/vmlinux-5.10.186" -o "$DIR/vmlinux-5.10.186"
  [ -f "$DIR/ubuntu-22.04.ext4" ] || curl -fsSL "$base/ubuntu-22.04.ext4" -o "$DIR/ubuntu-22.04.ext4"
  [ -f "$KEY" ] || { curl -fsSL "$base/ubuntu-22.04.id_rsa" -o "$KEY"; chmod 400 "$KEY"; }
  command -v firecracker >/dev/null 2>&1 || { echo "[fc-vm][ERR] firecracker absent (install.sh --provider l'installe)"; return 1; }
}

net_up() {
  out_iface="$(ip route show default | awk '{print $5; exit}')"
  ip link del "$TAP" 2>/dev/null || true
  ip tuntap add dev "$TAP" mode tap
  ip addr add "${TAP_IP}/30" dev "$TAP"
  ip link set dev "$TAP" up
  echo 1 > /proc/sys/net/ipv4/ip_forward
  iptables -t nat -D POSTROUTING -o "$out_iface" -j MASQUERADE 2>/dev/null || true
  iptables -D FORWARD -i "$TAP" -o "$out_iface" -j ACCEPT 2>/dev/null || true
  iptables -t nat -A POSTROUTING -o "$out_iface" -j MASQUERADE
  iptables -I FORWARD 1 -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
  iptables -I FORWARD 1 -i "$TAP" -o "$out_iface" -j ACCEPT
}

vm_config() {
  cat > "$DIR/vm.json" <<EOF
{
  "boot-source": {
    "kernel_image_path": "$DIR/vmlinux-5.10.186",
    "boot_args": "console=ttyS0 reboot=k panic=1 pci=off"
  },
  "drives": [
    {"drive_id": "rootfs", "path_on_host": "$DIR/ubuntu-22.04.ext4",
     "is_root_device": true, "is_read_only": false}
  ],
  "machine-config": {"vcpu_count": $VCPU, "mem_size_mib": $MEM, "smt": false},
  "network-interfaces": [
    {"iface_id": "net1", "guest_mac": "06:00:AC:10:00:02", "host_dev_name": "$TAP"}
  ]
}
EOF
}

wait_ssh() {
  for _ in $(seq 1 18); do
    $SSH true 2>/dev/null && return 0
    sleep 5
  done
  echo "[fc-vm][ERR] guest SSH injoignable apres 90s (voir /tmp/fc.log)" >&2
  return 1
}

cmd_up() {
  need curl; need iptables
  ensure_artifacts
  "$0" down >/dev/null 2>&1 || true
  net_up
  vm_config
  rm -f "$SOCK"
  setsid nohup firecracker --api-sock "$SOCK" --config-file "$DIR/vm.json" > /tmp/fc.log 2>&1 < /dev/null &
  info "boot ${VCPU}vCPU/${MEM}MB (log /tmp/fc.log)…"
  wait_ssh
  # Route par defaut volatile (perdue a chaque boot) : le guest sort via tap.
  $SSH 'ip route replace default via 172.16.0.1 dev eth0 2>/dev/null || ip route replace default via 172.16.0.1' || warn "route guest non posee"
  rotate_guest_key || warn "rotation cle guest echouee (cle CI gardee)"
  info "guest OK: $($SSH 'hostname; cat /proc/meminfo | head -1')"
}

cmd_down() {
  pkill -f "config-file $DIR/vm.json" 2>/dev/null || true
  sleep 1
  ip link del "$TAP" 2>/dev/null || true
  rm -f "$SOCK"
  info "vm arretee"
}

rotate_guest_key() {
  # Remplace la cle CI publique par une cle unique par VM.
  # Regenere si absente OU cassee (une cle cassee n'a jamais pu servir).
  local vmkey="$DIR/vm_key" newpub oldpub
  local SSH_NEW="ssh -i $vmkey -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=10 root@$GUEST_IP"
  if [ -f "$vmkey" ] && $SSH_NEW true 2>/dev/null; then
    info "cle guest unique deja active"
    return 0
  fi
  # Canal casse (ex: vm_key generee mais jamais injectee) : repli cle CI.
  if ! $SSH true 2>/dev/null; then
    SSH="ssh -i $DIR/ubuntu-22.04.id_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=10 root@$GUEST_IP"
    $SSH true 2>/dev/null || { echo "[fc-vm][ERR] aucun acces SSH guest" >&2; return 1; }
    info "repli cle CI pour bootstrap"
  fi
  rm -f "$vmkey" "$vmkey.pub" # ssh-keygen demanderait confirmation sinon
  ssh-keygen -t ed25519 -N "" -f "$vmkey" -q
  chmod 600 "$vmkey"
  newpub="$(cat "$vmkey.pub")"
  $SSH "grep -q -F '$newpub' ~/.ssh/authorized_keys 2>/dev/null || echo '$newpub' >> ~/.ssh/authorized_keys"
  $SSH_NEW true 2>/dev/null || { echo "[fc-vm][ERR] nouvelle cle refusee, CI gardee" >&2; return 1; }
  oldpub="$(ssh-keygen -y -f "$DIR/ubuntu-22.04.id_rsa")"
  $SSH "sed -i '\|$oldpub|d' ~/.ssh/authorized_keys"
  if $SSH true 2>/dev/null; then
    warn "cle CI toujours acceptee (non bloquant)"
  else
    info "cle CI revoquee, cle unique active"
  fi
}

cmd_resize() {  # Grow the rootfs image (VM must be down). Ex: fc-vm.sh resize +1536M
  # Inside the guest afterwards: resize2fs /dev/vda
  size="${1:?usage: fc-vm.sh resize +SIZE (ex: +1536M)}"
  if [ -S /tmp/fc-helium.socket ]; then
    echo "[fc-vm][ERR] arrete la VM d'abord (fc-vm.sh down)" >&2
    return 1
  fi
  img="$DIR/ubuntu-22.04.ext4"
  [ -f "$img" ] || { echo "[fc-vm][ERR] image absente: $img" >&2; return 1; }
  [ -f "$img.bak" ] || cp "$img" "$img.bak"
  qemu-img resize "$img" "$size"
  info "image agrandie, backup: $img.bak (dans le guest: resize2fs /dev/vda)"
}

cmd_expose() {
  gport="${1:?usage: fc-vm.sh expose <guest-port> [host-port]}"
  hport="${2:-$gport}"
  iptables -t nat -D PREROUTING -i "$WG_IF" -p tcp --dport "$hport" -j DNAT --to "$GUEST_IP:$gport" 2>/dev/null || true
  iptables -D FORWARD -i "$WG_IF" -o "$TAP" -p tcp -d "$GUEST_IP" --dport "$gport" -j ACCEPT 2>/dev/null || true
  iptables -t nat -D POSTROUTING -o "$TAP" -d "$GUEST_IP" -p tcp --dport "$gport" -j MASQUERADE 2>/dev/null || true
  iptables -t nat -A PREROUTING -i "$WG_IF" -p tcp --dport "$hport" -j DNAT --to "$GUEST_IP:$gport"
  iptables -I FORWARD 1 -i "$WG_IF" -o "$TAP" -p tcp -d "$GUEST_IP" --dport "$gport" -j ACCEPT
  iptables -t nat -A POSTROUTING -o "$TAP" -d "$GUEST_IP" -p tcp --dport "$gport" -j MASQUERADE
  info "expose: <tunnel>:$hport -> $GUEST_IP:$gport"
}

case "$CMD" in
  up) cmd_up ;;
  down) cmd_down ;;
  ssh) $SSH "$@" ;;
  expose) cmd_expose "$@" ;;
  resize) cmd_resize "$@" ;;
  rotate-key) rotate_guest_key ;;
  *) echo "usage: $0 {up|down|ssh|expose|resize|rotate-key} [options]"; exit 1 ;;
esac
