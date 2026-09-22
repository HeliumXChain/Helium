# Helium Benchmarks — measured, reproducible, no hype

All numbers measured on real hardware (see setup). Rerun commands included
so anyone can verify. No cloud, no simulation.

## Setup

| | Borrower (PC1) | Provider (PC2) | Guest microVM |
|---|---|---|---|
| OS | Windows 11 | Ubuntu Server 24.04, kernel 7.0 | Ubuntu 22.04, kernel 5.10.186 |
| CPU | desktop | 4 cores (laptop) | 2 vCPU |
| RAM | 16GB | 7GB | 2GB |
| Net | WiFi 5GHz, same /24 LAN | WiFi 5GHz | tap0 virtio |
| Build | helium v0.1.0-alpha16 | helium v0.1.0-alpha16 | python3 + numpy |

## Results (2026-09-21/22)

| Metric | Value | How |
|---|---|---|
| Tunnel ping (WireGuard) | **3ms avg** (2–6ms) | `ping -n 4 10.0.0.1` |
| Tunnel HTTP (guest page via DNAT) | **14ms** | `curl -w %{time_total} http://10.0.0.1:8888/` |
| Firecracker boot to SSH | **≤60s** | `fc-vm.sh up` → first successful SSH |
| P2P transfer, 50MB, hash-verified | **22.4s (~18 Mbps)** | `helium storage fetch` via tunnel, SHA-256 per chunk |
| ML training, 800 iters micro-MLP | **~3min**, loss 3.267→1.727 | `helium/scripts/mlp_train.py` in guest (2 vCPU) |
| mDNS first discovery | **~15s** | two fresh daemons, same LAN |
| Market match + settle | **<1s** (local SQLite) | `market match` + `accept` |
| Daemon auto-match cycle | **≤30s** after request | match loop interval |
| One-click install (warm) | **~2min** (apt path) | `install.sh --provider --daemon` |

## Honest limits

- **Throughput is WiFi-bound here** (~18 Mbps measured; WireGuard adds ~5%).
  Wired LAN or faster WiFi will do better. No datacenter claims.
- **No GPU passthrough** (host has GTX 850M, unused): training is CPU-only.
  GPU passthrough (vfio-pci) is future work, needs IOMMU + reboot.
- **Single VM per provider** for now (fixed tap0/socket); scheduler later.
- Guest rootfs is a 1.7GB Ubuntu (CI base + provisioned); PyTorch template pending.

## Reproduce

```bash
# tunnel
ping -n 4 10.0.0.1
curl -w 'total %{time_total}s\n' http://10.0.0.1:8888/
# transfer (needs provider token)
helium storage fetch --dataset <id> --from http://10.0.0.1:8788 --dir ./out --token <tok>
# training (inside guest)
python3 helium/scripts/mlp_train.py --iters 800
```
