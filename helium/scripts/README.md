# Helium MVP Technique - WireGuard POC Scripts

## Quick Test (2 Machines Required)

### Machine 1 - Provider (has GPU/resources)

```powershell
# Run as Administrator
.\wireguard-poc-test.ps1 -Role provider
```

Note the **Public Key** displayed. Share it with the borrower.

### Machine 2 - Borrower (needs GPU)

```powershell
# Run as Administrator
.\wireguard-poc-test.ps1 -Role borrower `
    -PeerEndpoint "PROVIDER_IP:51820" `
    -PeerPublicKey "PROVIDER_PUBLIC_KEY"
```

Replace:
- `PROVIDER_IP`: Public IP of provider machine (or local IP if same network)
- `PROVIDER_PUBLIC_KEY`: The key shown on provider machine

## What This Proves

✅ P2P tunnel establishment  
✅ Key exchange  
✅ NAT traversal (if using public IPs)  
✅ Bidirectional connectivity  

## Manual WireGuard Setup

If the script fails, manual steps:

### Generate Keys
```bash
wg genkey | tee private.key | wg pubkey > public.key
```

### Provider Config (`/etc/wireguard/helium.conf`)
```ini
[Interface]
PrivateKey = <provider-private-key>
Address = 10.0.0.1/24
ListenPort = 51820

[Peer]
PublicKey = <borrower-public-key>
AllowedIPs = 10.0.0.2/32
```

### Borrower Config
```ini
[Interface]
PrivateKey = <borrower-private-key>
Address = 10.0.0.2/24

[Peer]
PublicKey = <provider-public-key>
Endpoint = <provider-ip>:51820
AllowedIPs = 10.0.0.0/24
PersistentKeepalive = 25
```

### Start Tunnel
```bash
# Provider
sudo wg-quick up helium

# Borrower
sudo wg-quick up helium

# Test
ping 10.0.0.1  # From borrower
ping 10.0.0.2  # From provider
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "WireGuard not found" | Install from wireguard.com/install |
| "Access denied" | Run PowerShell as Administrator |
| "Cannot assign requested address" | Check if port 51820 is available |
| "No handshake" | Check firewall rules, verify public key |
| "Ping fails but handshake works" | Check AllowedIPs in config |

## Firewall Rules

Provider must allow inbound UDP on port 51820:

```powershell
# Windows (PowerShell Admin)
New-NetFirewallRule -DisplayName "Helium-WireGuard" -Direction Inbound -Protocol UDP -LocalPort 51820 -Action Allow
```

```bash
# Linux (iptables)
sudo iptables -A INPUT -p udp --dport 51820 -j ACCEPT

# Or ufw
sudo ufw allow 51820/udp
```

## Next Steps After POC

1. ✅ **Tunnel works** → Move to Firecracker VM test
2. ❌ **Tunnel fails** → Debug NAT/firewall, try relay fallback
3. 📝 **Document results** in `validation_evidence.md`

See `../validation_evidence.md` for recording results.
