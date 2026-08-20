# MVP Technique: WireGuard P2P Tunnel POC

## Objective

Validate that two machines can establish a secure P2P tunnel using WireGuard, forming the foundation of the Helium network layer.

**Success Criteria:**
- [ ] Tunnel established between provider and borrower
- [ ] Bidirectional ping works (10.0.0.1 ↔ 10.0.0.2)
- [ ] Keys exchanged correctly
- [ ] Connection stable for >5 minutes

## Prerequisites

### Provider Machine (GPU Owner)
- WireGuard installed
- Public IP or accessible via NAT (port forwarding)
- UDP port 51820 open in firewall

### Borrower Machine (GPU User)  
- WireGuard installed
- Internet connectivity

## Quick Start (Automated)

### Windows

```powershell
# Provider (Machine with GPU)
.\scripts\wireguard-poc-test.ps1 -Role provider

# Borrower (Machine needing GPU)
.\scripts\wireguard-poc-test.ps1 -Role borrower `
    -PeerEndpoint "PROVIDER_PUBLIC_IP:51820" `
    -PeerPublicKey "PROVIDER_PUBKEY_FROM_ABOVE"
```

### Linux/macOS

```bash
# Provider
sudo bash scripts/wireguard-poc-test.sh provider

# Borrower
bash scripts/wireguard-poc-test.sh borrower \
    "PROVIDER_PUBLIC_IP:51820" \
    "PROVIDER_PUBKEY"
```

## Manual Steps (If Automated Fails)

### Step 1: Generate Keys (Both Machines)

```bash
# Generate private key
wg genkey > private.key

# Generate public key
wg pubkey < private.key > public.key

# View keys
cat public.key   # Share this
cat private.key  # Keep secret
```

### Step 2: Provider Configuration

Create `/etc/wireguard/helium.conf`:

```ini
[Interface]
PrivateKey = <provider-private-key>
Address = 10.0.0.1/24
ListenPort = 51820

[Peer]
# Borrower's public key (they'll give you this)
PublicKey = <borrower-public-key>
AllowedIPs = 10.0.0.2/32
```

Start:
```bash
sudo wg-quick up helium
```

### Step 3: Borrower Configuration

Create `helium.conf`:

```ini
[Interface]
PrivateKey = <borrower-private-key>
Address = 10.0.0.2/24

[Peer]
# Provider's public key
PublicKey = <provider-public-key>
# Provider's public IP:port
Endpoint = 203.0.113.1:51820
AllowedIPs = 10.0.0.0/24
PersistentKeepalive = 25
```

Start:
```bash
sudo wg-quick up helium
```

### Step 4: Verify Connection

```bash
# From borrower
ping 10.0.0.1

# From provider  
ping 10.0.0.2

# Check handshake
wg show

# Should see:
# peer: <pubkey>
#   endpoint: <ip>:51820
#   latest handshake: 5 seconds ago
#   transfer: 1.21 MiB received, 2.43 MiB sent
```

## Troubleshooting

### No Handshake

```bash
# Check if provider is listening
sudo ss -tulnp | grep 51820

# Check firewall (provider)
sudo iptables -L | grep 51820

# Check NAT (if behind router)
# Provider needs port forwarding: 51820 UDP -> provider machine
```

### Handshake but No Ping

```bash
# Check AllowedIPs
wg show | grep -A5 "peer"

# Should include 10.0.0.x

# Check routes
ip route | grep 10.0.0
```

### Windows Specific

```powershell
# Run as Administrator
# Check if service is running
Get-Service -Name "WireGuardTunnel$*"

# Allow in Windows Firewall
New-NetFirewallRule -DisplayName "Helium-WG" -Direction Inbound -Protocol UDP -LocalPort 51820 -Action Allow
```

## Success Recording

After successful test, record in `validation_evidence.md`:

```markdown
## WireGuard POC Test - [Date]

**Provider**: [Machine spec, OS]
**Borrower**: [Machine spec, OS]
**Network**: [Same LAN / Internet / NAT]

### Results
- ✅ Tunnel established: [time]
- ✅ Provider IP: 10.0.0.1
- ✅ Borrower IP: 10.0.0.2  
- ✅ Ping latency: [X]ms
- ✅ Connection stable: [duration]

### Issues Encountered
- [None / List issues and solutions]

### Screenshots
- [Attach wg show output]
- [Attach ping test]
```

## Next: Firecracker VM Test

Once WireGuard works, proceed to test Firecracker VM creation over the tunnel.

See: `MVP_TECHNIQUE_FIRECRACKER_POC.md`
