# Tech Scout — Helium Communities (R69 Continuous)

**Date**: 2026-08-20
**Trigger**: S1 avant build + Technical HIGH (R14.5)
**Objectif**: Reduire Technical Risk en harvestant 3+ repos recents (<12m, >100 stars, last commit <30j)

---

## Blocage 1: NAT traversal + WireGuard mesh

### 1. netbirdio/netbird
- URL: https://github.com/netbirdio/netbird
- Stars: ~11k (active, 3229 commits, last 2026-08-12)
- License: BSD-3
- Ce qu'il resout: Mesh WireGuard complet avec ICE (pion/ice) + STUN + Signal Service + fallback Relay (TURN). Gere hole-punching et fallback automatique quand NAT strict.
- Limites: Go, pas Rust; besoin management/relay central si direct echoue
- Snippet: `Agents use ICE via pion/ice to discover candidates -> STUN -> Signal -> direct or Relay`
- Integration Helium: reutiliser pattern ICE/STUN/Relay pour notre TunnelManager, ou wrapper netbird client en sidecar pour MVP

### 2. wgmesh (atvirokodosprendimai/wgmesh)
- URL: https://github.com/atvirokodosprendimai/wgmesh + https://wgmesh.dev
- Stars: ~500-1k (2025-2026 active), MIT
- Ce qu'il resout: Exactement notre besoin — DHT peer discovery + WireGuard mesh decentralise, `wgmesh join --secret` derive swarm key, NAT hole-punch, zero central server, self-healing
- Limites: Go 1.23, requiert wg-tools kernel, pas de Firecracker integration
- Snippet: `wgmesh join --secret "wgmesh://v1/secret" -> DHT swarm -> WireGuard tunnels`
- Integration Helium: fork du DHT swarm logic pour notre CommunityDiscovery (remplace rendezvous), ou utiliser wgmesh comme tunnel layer MVP avant Rust natif

### 3. mullvad/gotatun (BoringTun fork Rust)
- URL: https://github.com/mullvad/gotatun
- Stars: 1300, Rust, MPL-2.0, 756 commits, active
- Ce qu'il resout: WireGuard userspace pur Rust (fork BoringTun) — pas besoin kernel module, marche Windows/macOS/Linux, ideal pour devs sans KVM/host privileges
- Limites: userspace = overhead CPU 10-20% vs kernel, pas de NAT traversal integre
- Snippet: `cargo build --bin gotatun --release` -> UAPI compatible WireGuard
- Integration Helium: fallback si WireGuard kernel non dispo (Windows provider), utiliser gotatun/boringtun dans TunnelManager

---

## Blocage 2: Firecracker microVM

### 4. firecracker-microvm/firecracker
- URL: https://github.com/firecracker-microvm/firecracker
- Stars: 35k, Rust, Apache-2.0, last push 2026-08-10 (active, 40 PRs open)
- Ce qu'il resout: MicroVM <125ms boot, 5MB overhead, 5 devices virtio, API REST Unix socket, battle-tested AWS Lambda/Fargate
- Limites: Requiert KVM + Linux 5.10+, pas Windows/macOS, rootfs ext4 build complexe
- Snippet: `curl --unix-socket /tmp/firecracker.sock http://localhost/machine-config -d '{"vcpu_count":2,"mem_size_mib":512}'`
- Integration Helium: notre helium-vm crate utilise firecracker-client 1.5, copy-paste API calls deja dans COMMUNITIES_ARCHITECTURE.md:293

### 5. firecracker-microvm/firecracker-containerd
- URL: https://github.com/firecracker-microvm/firecracker-containerd
- Stars: 2881, Go, active 2026-07-25
- Ce qu'il resout: containerd shim pour Firecracker — lance containers comme microVMs, simplifie rootfs (Docker export -> ext4)
- Limites: Go shim, pas Rust natif
- Snippet: `firecracker-containerd enables containerd to manage containers as Firecracker microVMs`
- Integration Helium: utiliser pour builder rootfs ML (Dockerfile -> ext4) sans custom script

---

## Blocage 3: DHT libp2p

### 6. libp2p/rust-libp2p (kad)
- URL: https://github.com/libp2p/rust-libp2p , docs https://docs.rs/libp2p-kad
- Stars: ~3k, Rust, active, Kademlia spec 2026-03
- Ce qu'il resout: Kademlia DHT mature, protocol `/helium/community/{id}/kad/1.0.0` isolation, Identify + Kademlia Behaviour, QUIC/Noise transports
- Limites: Besoin bootstrap nodes hardcodes, Identify must be hooked manually
- Snippet: `Kademlia::with_config(peer_id, store, config.set_protocol_names(vec![protocol.into()]))`
- Integration Helium: deja dans COMMUNITIES_ARCHITECTURE.md:51, code pret a tester avec 2 nodes

---

## Conclusion Tech Scout S1

**Evidence AGAINST Technical HIGH**:
- 3+ repos actifs (>1k stars, last commit <30j) resolvent chaque blocage separemment
- wgmesh prouve DHT+WireGuard mesh decentralise existe et marche (1 secret = mesh)
- netbird prouve NAT traversal + relay fallback robuste existe en prod
- firecracker prouve microVM <125ms existe et scale chez AWS

**Downgrade propose**: Technical HIGH -> MEDIUM (R69: 3+ repos trouves)
- Reste HIGH si on veut Windows provider (pas de KVM) ou NCCL over VPN (non prouve)
- Pour MVP Linux-only, 2 nodes, 1 VM, Technical = MEDIUM avec remedies ci-dessus

**Actions S1**:
1. Tester wgmesh en local comme reference (1 secret, 2 VMs) avant notre Rust impl
2. Choisir boringtun/gotatun pour userspace fallback si kernel non dispo
3. Builder rootfs via firecracker-containerd (Docker export) pour gagner 1j

**Next harvest**: S2 avant Firecracker integration (harvester NCCL, cloud-hypervisor)
