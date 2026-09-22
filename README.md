# Helium — private P2P GPU sharing for people who trust each other

Borrow your friend's idle GPU over an encrypted tunnel, run the workload in
an isolated microVM, pay in local credits. No marketplace strangers, no
blockchain, no wallet. MIT open source.

**Status**: technical MVP validated end-to-end (tunnel + microVM + market +
auto-matching + TUI). Binary `v0.1.0-alpha16`: Linux (x86_64/aarch64), macOS,
Windows + `.deb`.

![Helium demo](docs/demo.gif)

```
borrower --(WireGuard)-- provider --(tap)--> Firecracker microVM
   |                           |
 CLI / TUI / daemon     SQLite market + credits
   |                           |
 mDNS (LAN) + Kademlia (WAN) discovery
```

## Quick start

**Provider** (shares RAM/GPU, Linux with KVM):

```bash
curl -fsSL https://raw.githubusercontent.com/HeliumXChain/Helium/main/install.sh | sudo bash -s -- --provider --daemon
```

**Borrower** (needs compute):

```bash
# Linux
curl -fsSL https://raw.githubusercontent.com/HeliumXChain/Helium/main/install.sh | bash -s -- --borrower --endpoint PROVIDER_IP:51820 --pubkey PROVIDER_KEY
# Windows (admin PowerShell)
./install.ps1 -Borrower -Endpoint "PROVIDER_IP:51820" -PeerPubKey "PROVIDER_KEY"
```

**Marketplace** (either side):

```bash
helium faucet --amount 100                      # demo credits
helium market offer --rtype gpu --amount 32 --price 0.5
helium market request --rtype gpu --amount 16 --max-price 1.0 --hours 2
helium market match --request-id <id> && helium market accept --match-id <id>
helium daemon                                   # auto-matches every 30s
helium dash                                     # fullscreen dashboard
```

Requirements: Linux 5.10+, `/dev/kvm` (provider), 8GB RAM recommended.

## Layout

```
helium-mesh/          # the shipped binary (Rust): CLI, daemon, market, tunnel, discovery, VM, TUI
helium/crates/        # original workspace (core matcher, tunnel/VM stubs, CLI)
helium/scripts/       # fc-vm.sh (microVM boot/ssh/expose), fc-provision.sh (guest setup), mlp_train.py (demo)
install.sh            # one-click Linux installer (apt-first, tunnel, firecracker, daemon service)
install.ps1           # Windows installer (binary, PATH, firewall, borrower tunnel)
```

## Validation

Every gate below ran on real hardware (Windows borrower + Ubuntu provider),
logged in `validation_evidence.md` (private):

- WireGuard tunnel, ping 2–3ms, survives reboots
- Firecracker boot <1min + SSH, ML training inside the VM (loss 3.27→1.73)
- 2 manual credit-settled transactions + daemon auto-matching + auto tunnel provisioning
- mDNS + Kademlia discovery with stable peer IDs, 300MB peer-to-peer transfer hash-verified

## License

MIT. Issues and PRs welcome after v0.1 — validation by usage (stars, forks), not interviews.
