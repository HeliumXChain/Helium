#!/bin/bash
# WireGuard POC Test Script (Linux/macOS)
# Tests P2P tunnel establishment between two Helium nodes

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

info() { echo -e "${CYAN}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
error() { echo -e "${RED}[ERR]${NC} $1"; }
step() { echo -e "\n${YELLOW}>>${NC} $1"; }

# Check arguments
ROLE=$1
if [ -z "$ROLE" ] || [[ ! "$ROLE" =~ ^(provider|borrower)$ ]]; then
    error "Usage: $0 <provider|borrower> [peer_endpoint] [peer_pubkey]"
    exit 1
fi

PEER_ENDPOINT=$2
PEER_PUBKEY=$3

# Check WireGuard
step "Checking WireGuard installation..."
if command -v wg &> /dev/null; then
    success "WireGuard found: $(wg --version)"
else
    error "WireGuard not installed"
    info "Install: sudo apt install wireguard-tools  # Debian/Ubuntu"
    info "Install: sudo yum install wireguard-tools   # RHEL/CentOS"
    info "Install: brew install wireguard-tools       # macOS"
    exit 1
fi

# Check if running as root for provider
if [ "$ROLE" = "provider" ] && [ "$EUID" -ne 0 ]; then 
    error "Provider role must run as root (for binding to port 51820)"
    exit 1
fi

# Generate keys
step "Generating WireGuard keypair..."
PRIVATE_KEY=$(wg genkey)
PUBLIC_KEY=$(echo "$PRIVATE_KEY" | wg pubkey)

info "Public Key: $PUBLIC_KEY"
info "Private Key: ${PRIVATE_KEY:0:8}..."

# Configuration
INTERFACE_NAME="helium-poc"
LISTEN_PORT=51820

if [ "$ROLE" = "provider" ]; then
    step "Configuring as PROVIDER (listening on port $LISTEN_PORT)..."
    
    cat > /tmp/helium-poc.conf << EOF
[Interface]
PrivateKey = $PRIVATE_KEY
Address = 10.0.0.1/24
ListenPort = $LISTEN_PORT
EOF
    
    success "Provider config generated"
    info "Your PUBLIC KEY: $PUBLIC_KEY"
    info "Share this with the borrower"
    
else
    if [ -z "$PEER_ENDPOINT" ] || [ -z "$PEER_PUBKEY" ]; then
        error "Borrower requires PEER_ENDPOINT and PEER_PUBKEY"
        info "Example: $0 borrower '192.168.1.100:51820' 'abcd1234...'"
        exit 1
    fi
    
    step "Configuring as BORROWER (connecting to $PEER_ENDPOINT)..."
    
    cat > /tmp/helium-poc.conf << EOF
[Interface]
PrivateKey = $PRIVATE_KEY
Address = 10.0.0.2/24

[Peer]
PublicKey = $PEER_PUBKEY
Endpoint = $PEER_ENDPOINT
AllowedIPs = 10.0.0.0/24
PersistentKeepalive = 25
EOF
    
    success "Borrower config generated"
fi

# Display config
echo -e "\n${MAGENTA}=== WireGuard Configuration ===${NC}"
cat /tmp/helium-poc.conf
echo -e "${MAGENTA}=================================${NC}\n"

# Setup tunnel
step "Creating tunnel interface..."

# Remove existing if present
wg-quick down "$INTERFACE_NAME" 2>/dev/null || true
ip link del "$INTERFACE_NAME" 2>/dev/null || true

# Create interface
if [ "$ROLE" = "provider" ]; then
    ip link add dev "$INTERFACE_NAME" type wireguard
    ip address add dev "$INTERFACE_NAME" 10.0.0.1/24
    wg set "$INTERFACE_NAME" listen-port $LISTEN_PORT private-key <(echo "$PRIVATE_KEY")
else
    ip link add dev "$INTERFACE_NAME" type wireguard
    ip address add dev "$INTERFACE_NAME" 10.0.0.2/24
    wg set "$INTERFACE_NAME" private-key <(echo "$PRIVATE_KEY")
    wg set "$INTERFACE_NAME" peer "$PEER_PUBKEY" endpoint "$PEER_ENDPOINT" allowed-ips 10.0.0.0/24 persistent-keepalive 25
fi

ip link set up dev "$INTERFACE_NAME"

success "Tunnel interface '$INTERFACE_NAME' created!"

# Show status
step "Interface Status:"
wg show "$INTERFACE_NAME"

# Test connectivity
if [ "$ROLE" = "borrower" ]; then
    step "Testing connectivity to provider (10.0.0.1)..."
    sleep 2
    
    if ping -c 3 -W 5 10.0.0.1 &> /dev/null; then
        success "Ping successful! Tunnel is working."
        ping -c 1 -W 2 10.0.0.1 | grep "time=" | head -1
    else
        error "Ping failed"
        info "Check: wg show"
        info "Check firewall rules on provider"
    fi
fi

echo -e "\n${GREEN}=== SUCCESS ===${NC}"
echo -e "WireGuard tunnel established!"
echo -e "Interface: ${CYAN}$INTERFACE_NAME${NC}"
echo -e "Your IP: ${CYAN}$(if [ '$ROLE' = 'provider' ]; then echo '10.0.0.1'; else echo '10.0.0.2'; fi)${NC}"

if [ "$ROLE" = "provider" ]; then
    echo -e "\n${YELLOW}Next steps for borrower:${NC}"
    echo "1. Share your public key: $PUBLIC_KEY"
    PUBLIC_IP=$(curl -s https://api.ipify.org 2>/dev/null || echo "YOUR_PUBLIC_IP")
    echo "2. Share your public IP: $PUBLIC_IP"
    echo "3. Ensure firewall allows UDP port $LISTEN_PORT"
fi

# Monitor
echo -e "\n${YELLOW}=======================================${NC}"
echo -e "${YELLOW}TUNNEL IS ACTIVE - Press Ctrl+C to stop${NC}"
echo -e "${YELLOW}=======================================${NC}\n"

cleanup() {
    echo -e "\n${YELLOW}Shutting down tunnel...${NC}"
    wg-quick down "$INTERFACE_NAME" 2>/dev/null || true
    ip link del "$INTERFACE_NAME" 2>/dev/null || true
    success "Tunnel closed"
    exit 0
}

trap cleanup INT TERM

# Monitor handshake
while true; do
    HANDSHAKE=$(wg show "$INTERFACE_NAME" latest-handshakes 2>/dev/null | head -1)
    if [ -n "$HANDSHAKE" ]; then
        echo -e "$(date '+%H:%M:%S') - ${GRAY}$HANDSHAKE${NC}"
    fi
    sleep 5
done
