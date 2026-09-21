#!/usr/bin/env bash
# Helium one-click installer + node calibration (S4 — PLAN.md #6)
# Usage:
#   curl -fsSL https://helium.network/install.sh | sh
#   curl -fsSL https://helium.network/install.sh | sh -s -- v0.1.0
#   sh install.sh --check-only        # calibrage seul, sans install
#   sh install.sh --source            # build depuis sources (cargo)
#   sh install.sh --dir ~/.local/bin  # dossier d'install custom
#   sudo sh install.sh --provider --daemon   # provider complet + service
#   sh install.sh --borrower --endpoint IP:51820 --pubkey CLE
set -euo pipefail

REPO="${HELIUM_REPO:-HeliumXChain/Helium}"
BINARY="helium"
PKG="helium-mesh"
MIN_KERNEL_MAJOR=5
MIN_KERNEL_MINOR=10
MIN_RAM_GB=8

VERSION=""
case "${1:-}" in
  v*|0.*) VERSION="$1" ;;
esac
INSTALL_DIR=""
CHECK_ONLY=0
FROM_SOURCE=0
WITH_DAEMON=0
ROLE=""
ENDPOINT=""
PEER_PUBKEY=""
FIRECRACKER_VERSION="1.5.1"
WG_IF="helium-poc"
WG_PORT="51820"

for arg in "$@"; do
  case "$arg" in
    --check-only) CHECK_ONLY=1 ;;
    --source) FROM_SOURCE=1 ;;
    --daemon) WITH_DAEMON=1 ;;
    --provider) ROLE="provider" ;;
    --borrower) ROLE="borrower" ;;
    --endpoint|--pubkey|--endpoint=*|--pubkey=*) ;;
    --dir) shift ;;
    --dir=*) INSTALL_DIR="${arg#--dir=}" ;;
    v*|0.*) VERSION="$arg" ;;
  esac
done
# second pass for options taking a separate value
prev=""
for arg in "$@"; do
  case "$prev" in
    --dir) INSTALL_DIR="$arg" ;;
    --endpoint) ENDPOINT="$arg" ;;
    --pubkey) PEER_PUBKEY="$arg" ;;
  esac
  if [[ "$arg" == --endpoint=* ]]; then ENDPOINT="${arg#--endpoint=}"; fi
  if [[ "$arg" == --pubkey=* ]]; then PEER_PUBKEY="${arg#--pubkey=}"; fi
  prev="$arg"
done

info()  { printf '\033[0;36m[INFO]\033[0m %s\n' "$*"; }
ok()    { printf '\033[0;32m[OK]\033[0m %s\n' "$*"; }
warn()  { printf '\033[1;33m[WARN]\033[0m %s\n' "$*"; }
err()   { printf '\033[0;31m[ERR]\033[0m %s\n' "$*" >&2; }

need() { command -v "$1" >/dev/null 2>&1 || { err "missing: $1"; return 1; }; }

detect_platform() {
  OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
  ARCH="$(uname -m)"
  case "$OS" in
    linux)  PLAT_OS="linux" ;;
    darwin) PLAT_OS="macos" ;;
    mingw*|msys*|cygwin*) PLAT_OS="windows" ;;
    *) err "OS non supporte: $OS (linux/macOS supportes, Windows via WSL)"; exit 1 ;;
  esac
  case "$ARCH" in
    x86_64|amd64) PLAT_ARCH="x86_64" ;;
    aarch64|arm64) PLAT_ARCH="aarch64" ;;
    *) err "arch non supportee: $ARCH"; exit 1 ;;
  esac
  info "plateforme: $PLAT_OS/$PLAT_ARCH"
}

latest_version() {
  # $1 = repo "org/name" -> "vX.Y.Z" ou vide si API injoignable
  curl -fsSL "https://api.github.com/repos/$1/releases/latest" 2>/dev/null \
    | grep -m1 '"tag_name"' | cut -d'"' -f4 || true
}

pick_install_dir() {
  if [ -n "$INSTALL_DIR" ]; then return 0; fi
  if [ -w "/usr/local/bin" ] 2>/dev/null || [ "$(id -u)" = "0" ]; then
    INSTALL_DIR="/usr/local/bin"
  else
    INSTALL_DIR="$HOME/.local/bin"
  fi
}

install_system_deps() {
  # WireGuard + curl/tar, best-effort selon gestionnaire de paquets
  if command -v wg >/dev/null 2>&1; then ok "wireguard-tools present: $(wg --version)"; return 0; fi
  warn "wireguard-tools absent — tentative d'installation auto"
  if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update -qq && sudo apt-get install -y -qq wireguard-tools curl tar 2>&1 | tail -2 || warn "apt auto-install a echoue, installe wireguard-tools a la main"
  elif command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y wireguard-tools curl tar || warn "dnf auto-install a echoue"
  elif command -v pacman >/dev/null 2>&1; then
    sudo pacman -Sy --noconfirm wireguard-tools curl tar || warn "pacman auto-install a echoue"
  elif command -v brew >/dev/null 2>&1; then
    brew install wireguard-tools || warn "brew auto-install a echoue"
  else
    warn "gestionnaire de paquets inconnu — installe wireguard-tools manuellement: https://www.wireguard.com/install/"
  fi
}

install_from_release() {
  ver="$1"
  asset="${BINARY}-${ver}-${PLAT_OS}-${PLAT_ARCH}.tar.gz"
  url="https://github.com/${REPO}/releases/download/${ver}/${asset}"
  info "telechargement: $url"
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  if ! curl -fsSL "$url" -o "$tmp/helium.tar.gz"; then
    err "telechargement impossible ($asset). Release absente ? Essaie --source."
    return 1
  fi
  tar -xzf "$tmp/helium.tar.gz" -C "$tmp"
  bin_path="$(find "$tmp" -name "$BINARY" -type f | head -1)"
  [ -n "$bin_path" ] || { err "binaire $BINARY introuvable dans l'archive"; return 1; }
  mkdir -p "$INSTALL_DIR"
  if [ -w "$INSTALL_DIR" ]; then cp "$bin_path" "$INSTALL_DIR/$BINARY"; chmod +x "$INSTALL_DIR/$BINARY";
  else sudo cp "$bin_path" "$INSTALL_DIR/$BINARY"; fi
  ok "installe: $INSTALL_DIR/$BINARY ($ver)"
  rm -rf "$tmp"; trap - EXIT
}

ensure_rust() {
  command -v cargo >/dev/null 2>&1 && cargo --version >/dev/null 2>&1 && return 0
  for d in "$HOME/.cargo/bin" /root/.cargo/bin /home/*/.cargo/bin; do
    if [ -x "$d/cargo" ] && "$d/cargo" --version >/dev/null 2>&1; then
      export PATH="$d:$PATH"; return 0
    fi
  done
  if command -v rustup >/dev/null 2>&1; then
    info "installation toolchain Rust stable…"
    rustup toolchain install stable >/dev/null 2>&1 || true
    command -v cargo >/dev/null 2>&1 && cargo --version >/dev/null 2>&1 && return 0
  fi
  info "installation Rust (rustup, profil minimal)…"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable >/dev/null 2>&1 || return 1
  export PATH="$HOME/.cargo/bin:$PATH"
  cargo --version >/dev/null 2>&1
}

install_from_source() {
  ensure_rust || { err "cargo absent — installe Rust: https://rustup.rs puis relance avec --source"; exit 1; }
  if command -v dpkg >/dev/null 2>&1 && dpkg -l helium >/dev/null 2>&1; then
    warn "paquet apt 'helium' deja installe (/usr/bin) — prefere 'apt upgrade' ou accepte l'ombrage via $INSTALL_DIR"
  fi
  info "build depuis sources…"
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  build_args="--release -p $PKG"
  if [ -f "./helium-mesh/Cargo.toml" ]; then
    src="$PWD"  # racine du repo
  elif [ -f "./Cargo.toml" ]; then
    src="$PWD"; build_args="--release"  # dossier du crate directement
  else
    need git || { err "git requis pour cloner les sources"; exit 1; }
    git clone --depth 1 "https://github.com/${REPO}.git" "$tmp/src" && src="$tmp/src"
  fi
  (cd "$src" && cargo build $build_args)
  bin_path="$src/target/release/$BINARY"
  [ -f "$bin_path" ] || { err "build OK mais binaire introuvable: $bin_path"; exit 1; }
  mkdir -p "$INSTALL_DIR"
  if [ -w "$INSTALL_DIR" ]; then cp "$bin_path" "$INSTALL_DIR/$BINARY";
  else sudo cp "$bin_path" "$INSTALL_DIR/$BINARY"; fi
  chmod +x "$INSTALL_DIR/$BINARY"
  ok "installe depuis sources: $INSTALL_DIR/$BINARY"
  rm -rf "$tmp"; trap - EXIT
}

SUDO=""
if [ "$(id -u)" != "0" ]; then SUDO="sudo"; fi

wg_enable_service() {
  # $1 = interface (ex helium-poc). Active wg-quick@ au boot (paquet `wireguard` requis pour l'unite).
  if command -v apt-get >/dev/null 2>&1; then
    $SUDO apt-get install -y -qq wireguard 2>&1 | tail -1 || true
  fi
  $SUDO systemctl enable --now "wg-quick@$1"
}

# ---- tunnel WireGuard persistant ----
setup_wg_sudoers() {
  # Laisse le daemon (non-root) gerer les peers wg sans mot de passe.
  local wg_bin rule_user
  wg_bin="$(command -v wg || true)"
  rule_user="${SUDO_USER:-$(id -un)}"
  [ -n "$wg_bin" ] && [ "$rule_user" != "root" ] || return 0
  echo "$rule_user ALL=(root) NOPASSWD: $wg_bin" | $SUDO tee /etc/sudoers.d/helium-wg >/dev/null
  $SUDO chmod 440 /etc/sudoers.d/helium-wg
  if $SUDO visudo -c -q 2>/dev/null; then
    ok "sudoers: $rule_user gere wg sans mot de passe"
  else
    warn "sudoers invalide, rollback"
    $SUDO rm -f /etc/sudoers.d/helium-wg
    return 1
  fi
}
setup_tunnel_provider() {
  [ "$PLAT_OS" = "linux" ] || { err "role provider persistant = Linux requis"; return 1; }
  command -v wg >/dev/null 2>&1 || { err "wireguard-tools absent"; return 1; }
  conf="/etc/wireguard/$WG_IF.conf"
  if [ -f "$conf" ]; then
    ok "config existante gardee: $conf (cles inchangees)"
    pub="$($SUDO wg show "$WG_IF" public-key 2>/dev/null || true)"
  else
    info "generation cle provider…"
    priv="$(wg genkey)"
    pub="$(echo "$priv" | wg pubkey)"
    $SUDO install -d -m 0755 /etc/wireguard
    echo "$priv" | $SUDO tee /etc/wireguard/$WG_IF.key >/dev/null
    $SUDO chmod 600 /etc/wireguard/$WG_IF.key
    {
      echo "[Interface]"
      echo "PrivateKey = $priv"
      echo "Address = 10.0.0.1/24"
      echo "ListenPort = $WG_PORT"
      if [ -n "$PEER_PUBKEY" ]; then
        echo ""
        echo "[Peer]"
        echo "PublicKey = $PEER_PUBKEY"
        echo "AllowedIPs = 10.0.0.2/32"
      fi
    } | $SUDO tee "$conf" >/dev/null
    $SUDO chmod 600 "$conf"
    [ -f "$conf" ] || { err "ecriture impossible: $conf (droits sudo ?)"; return 1; }
    ok "config ecrite: $conf"
  fi
  ip link show "$WG_IF" >/dev/null 2>&1 && $SUDO ip link del "$WG_IF" 2>/dev/null || true
  wg_enable_service "$WG_IF"
  $SUDO systemctl is-active --quiet "wg-quick@$WG_IF" || { err "service wg-quick@$WG_IF inactif"; return 1; }
  setup_wg_sudoers || true
  ok "tunnel provider actif: 10.0.0.1/24 (port $WG_PORT)"
  echo "PROVIDER-PUBKEY: $pub"
  if [ -z "$PEER_PUBKEY" ]; then
    warn "aucun borrower declare — ajout ulterieur: wg set $WG_IF peer <CLE_BORROWER> allowed-ips 10.0.0.2/32"
  fi
  echo "Borrower: sh install.sh --borrower --endpoint <IP_LAN>:$WG_PORT --pubkey $pub"
}

setup_tunnel_borrower() {
  [ -n "$ENDPOINT" ] && [ -n "$PEER_PUBKEY" ] || { err "borrower requiert --endpoint IP:PORT et --pubkey CLE"; return 1; }
  command -v wg >/dev/null 2>&1 || { err "wireguard-tools absent"; return 1; }
  priv="$(wg genkey)"
  pub="$(echo "$priv" | wg pubkey)"
  if [ "$PLAT_OS" = "linux" ]; then
    conf="/etc/wireguard/$WG_IF.conf"
    $SUDO install -d -m 0755 /etc/wireguard
    {
      echo "[Interface]"
      echo "PrivateKey = $priv"
      echo "Address = 10.0.0.2/24"
      echo ""
      echo "[Peer]"
      echo "PublicKey = $PEER_PUBKEY"
      echo "Endpoint = $ENDPOINT"
      echo "AllowedIPs = 10.0.0.0/24"
      echo "PersistentKeepalive = 25"
    } | $SUDO tee "$conf" >/dev/null
    $SUDO chmod 600 "$conf"
    [ -f "$conf" ] || { err "ecriture impossible: $conf (droits sudo ?)"; return 1; }
    ip link show "$WG_IF" >/dev/null 2>&1 && $SUDO ip link del "$WG_IF" 2>/dev/null || true
    wg_enable_service "$WG_IF"
    $SUDO systemctl is-active --quiet "wg-quick@$WG_IF" || { err "service wg-quick@$WG_IF inactif"; return 1; }
    setup_wg_sudoers || true
    ok "tunnel borrower actif: 10.0.0.2/24 -> $ENDPOINT"
  else
    warn "setup auto borrower = Linux uniquement — config manuelle:"
    echo "PrivateKey: $priv"
    echo "Endpoint: $ENDPOINT / Peer: $PEER_PUBKEY"
  fi
  echo "BORROWER-PUBKEY: $pub"
  echo "Sur provider: wg set $WG_IF peer $pub allowed-ips 10.0.0.2/32"
}

install_firecracker() {
  [ "$PLAT_OS" = "linux" ] || return 0
  if command -v firecracker >/dev/null 2>&1; then ok "firecracker present: $(firecracker --version | head -1)"; return 0; fi
  case "$PLAT_ARCH" in
    x86_64) fc_arch="x86_64" ;;
    aarch64) fc_arch="aarch64" ;;
  esac
  info "installation firecracker v$FIRECRACKER_VERSION…"
  tmp="$(mktemp -d)"
  url="https://github.com/firecracker-microvm/firecracker/releases/download/v${FIRECRACKER_VERSION}/firecracker-v${FIRECRACKER_VERSION}-${fc_arch}.tgz"
  curl -fsSL "$url" -o "$tmp/fc.tgz" || { err "telechargement firecracker impossible"; rm -rf "$tmp"; return 1; }
  tar -xzf "$tmp/fc.tgz" -C "$tmp"
  fc_bin="$(find "$tmp" -name "firecracker-v${FIRECRACKER_VERSION}-${fc_arch}" -type f | head -1)"
  dest="/usr/local/bin"
  if [ -w "$dest" ]; then cp "$fc_bin" "$dest/firecracker";
  else $SUDO cp "$fc_bin" "$dest/firecracker"; fi
  rm -rf "$tmp"
  firecracker --version | head -1 && ok "firecracker installe"
}

# ---- calibrage noeud (gate S1, sans ecrire hors /tmp) ----
calibrate() {
  info "calibrage noeud Helium…"
  fails=0

  # 1. kernel >= 5.10 (Linux uniquement, Firecracker/KVM)
  if [ "$PLAT_OS" = "linux" ]; then
    krel="$(uname -r)"
    major="$(echo "$krel" | cut -d. -f1)"; minor="$(echo "$krel" | cut -d. -f2)"
    if [ "$major" -gt "$MIN_KERNEL_MAJOR" ] || { [ "$major" -eq "$MIN_KERNEL_MAJOR" ] && [ "$minor" -ge "$MIN_KERNEL_MINOR" ]; }; then
      ok "kernel $krel (>= 5.10)"
    else
      err "kernel $krel < 5.10 — upgrade requis pour Firecracker"; fails=$((fails+1))
    fi
    # 2. KVM
    if [ -e /dev/kvm ]; then ok "/dev/kvm present (VM possible)";
    else warn "/dev/kvm absent — active VT-x/AMD-V en BIOS, sinon tunnel seul (fallback SSH+Docker)"; fi
    # 3. RAM
    if command -v free >/dev/null 2>&1; then
      ram_gb="$(free -g | awk '/^Mem:/{print $2}')"
      if [ "${ram_gb:-0}" -ge "$MIN_RAM_GB" ]; then ok "RAM ${ram_gb}GB (>= 8GB provider)";
      else warn "RAM ${ram_gb}GB < 8GB — 1 VM 2GB max pour le POC (template 4GB deconseille)"; fi
    fi
    # 4. port WireGuard libre
    if command -v ss >/dev/null 2>&1 && ss -uln 2>/dev/null | grep -q ":51820 "; then
      warn "UDP 51820 deja ecoute — un autre WireGuard tourne ?"
    else ok "UDP 51820 libre"; fi
  else
    warn "calibrage KVM ignore sur $PLAT_OS (provider = Linux requis, ce noeud = borrower OK)"
  fi

  # 5. wireguard
  if command -v wg >/dev/null 2>&1; then ok "wireguard: $(wg --version)";
  else warn "wireguard-tools absent — tunnel impossible sans lui"; fails=$((fails+1)); fi
  # 6. reseau
  if getent hosts github.com >/dev/null 2>&1 || ping -c1 -W2 1.1.1.1 >/dev/null 2>&1; then ok "reseau OK";
  else warn "pas de reseau — releases GitHub injoignables"; fi
  # 7. disque
  if [ "$(df -m . 2>/dev/null | awk 'NR==2{print $4}')" -lt 2048 ] 2>/dev/null; then warn "disque < 2GB libres";
  else ok "disque OK"; fi

  if [ "$fails" -gt 0 ]; then err "calibrage: $fails check(s) bloquant(s)"; return 1; fi
  ok "calibrage OK — pret pour POC tunnel"
}

print_next_steps() {
  echo ""
  echo "=== Next steps (POC S1) ==="
  echo "Provider (ce noeud si Linux) :  sudo $BINARY status   # puis scripts/wireguard-poc-test.sh provider"
  echo "Borrower (depuis l'autre PC) :  PeerEndpoint \"<IP_LAN>:51820\" + PublicKey du provider"
  echo "Succes = ping 10.0.0.1 OK. Log le resultat dans validation_evidence.md"
  case ":$PATH:" in *":$INSTALL_DIR:"*) ;; *) warn "$INSTALL_DIR hors PATH — ajoute: export PATH=\"\$PATH:$INSTALL_DIR\""; esac
}

main() {
  detect_platform
  if [ "$CHECK_ONLY" = "1" ]; then calibrate; exit $?; fi
  pick_install_dir
  info "install dir: $INSTALL_DIR"
  need curl || { err "curl requis"; exit 1; }
  need tar  || { err "tar requis"; exit 1; }
  install_system_deps || true
  install_helium_binary
  calibrate || warn "installe mais calibrage partiel — voir warnings ci-dessus"
  if [ "$ROLE" = "provider" ]; then
    setup_tunnel_provider || warn "setup tunnel provider partiel"
    install_firecracker || warn "firecracker non installe"
  elif [ "$ROLE" = "borrower" ]; then
    setup_tunnel_borrower || warn "setup tunnel borrower partiel"
  fi
  if [ "$WITH_DAEMON" = "1" ]; then
    install_daemon_service || warn "service daemon non installe"
  fi
  if command -v "$BINARY" >/dev/null 2>&1; then
    "$BINARY" --version 2>/dev/null || true
  elif [ -x "$INSTALL_DIR/$BINARY" ]; then
    "$INSTALL_DIR/$BINARY" --version 2>/dev/null || true
  fi
  ok "Helium pret"
  print_next_steps
}

# Ordre: apt (si repo configure) -> release GitHub -> build source.
install_helium_binary() {
  if [ "$FROM_SOURCE" != "1" ] \
    && command -v apt-get >/dev/null 2>&1 \
    && ls /etc/apt/sources.list.d/helium*.list >/dev/null 2>&1; then
    info "installation via apt…"
    $SUDO apt-get update -qq 2>&1 | tail -1 || true
    if $SUDO apt-get install -y -qq helium 2>&1 | tail -2; then
      ok "helium installe via apt"
      return 0
    fi
    warn "apt a echoue, essai release GitHub"
  fi
  if [ "$FROM_SOURCE" = "1" ]; then install_from_source; return 0; fi
  if [ -z "$VERSION" ]; then VERSION="$(latest_version "$REPO")"; fi
  if [ -z "$VERSION" ]; then warn "derniere release introuvable — fallback --source"; install_from_source;
  else install_from_release "$VERSION" || { warn "fallback build source"; install_from_source; }; fi
}

# Service systemd pour `helium daemon` (redemarrage auto).
install_daemon_service() {
  [ "$PLAT_OS" = "linux" ] || { err "service daemon = Linux (systemd) requis"; return 1; }
  command -v systemctl >/dev/null 2>&1 || { err "systemctl absent"; return 1; }
  bin="$(command -v "$BINARY" || echo "$INSTALL_DIR/$BINARY")"
  [ -x "$bin" ] || { err "binaire $BINARY introuvable"; return 1; }
  daemon_user="${SUDO_USER:-$(id -un)}"
  [ "$daemon_user" = "root" ] && daemon_user="root"
  $SUDO tee /etc/systemd/system/helium-daemon.service >/dev/null <<EOF
[Unit]
Description=Helium Mesh daemon (match auto + discovery)
After=network-online.target wg-quick@helium-poc.service
Wants=network-online.target
PartOf=wg-quick@helium-poc.service

[Service]
Type=simple
User=$daemon_user
ExecStart=$bin daemon
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
  $SUDO systemctl daemon-reload
  $SUDO systemctl enable --now helium-daemon
  sleep 2
  $SUDO systemctl is-active --quiet helium-daemon || { err "service helium-daemon inactif"; return 1; }
  ok "service helium-daemon actif (user $daemon_user, restart auto)"
}

main "$@"
