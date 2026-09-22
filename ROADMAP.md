# ROADMAP.md — Helium Communities (PUBLIC)

**Vision**: Reseaux P2P prives pour partager RAM/GPU entre devs de confiance. Tunnel chiffre WireGuard + microVM Firecracker isolee + credits locaux. Alternative simple a Vast.ai (strangers) et AWS (10x cher) pour le cas "besoin de 32GB pour fine-tuner, un ami a 64GB idle".

**Status**: MVP Technique VALIDE — gates S1 (tunnel P2P), S2 (boot microVM), S3 (2 transactions market) passes. Binaire `v0.1.0-alpha13` : Linux x86_64/aarch64, macOS, Windows + .deb. Prochaine etape : v0.2 mesh persistant.
**License**: MIT (prevu)
**Stack**: Rust, libp2p, WireGuard, Firecracker, SQLite, Tokio

---

## Architecture (public-friendly)

```
CLI (Rust) -> Discovery (libp2p DHT privee) -> Matching (SQLite) -> Tunnel (WireGuard) -> VM (Firecracker)
```

- DHT privee par communaute (isolation)
- Tunnel chiffre ChaCha20-Poly1305
- VM ephemere KVM, rootfs read-only
- Credits locaux (pas de blockchain en MVP)

---

## Version Roadmap

### v0.1 — MVP Technique (VALIDE 2026-09-21)

- [x] Workspace Rust 4 crates (core/tunnel/vm/cli) + binaire `helium-mesh` unifie
- [x] Discovery mDNS zero-config + Kademlia (IDs stables)
- [x] WireGuard tunnel P2P 2 nodes (ping 10.0.0.1 3/3, cross-OS Windows<->Linux)
- [x] Firecracker microVM boot < 1min + SSH (2vCPU/2Go)
- [x] Tunnel + VM integres (HTTP servi depuis la VM via tunnel)
- [x] Matching + credits SQLite (2 transactions manuelles, scoring prouve)
- [x] CLI `init/status/market/tunnel/mesh/daemon/storage` + TUI `dash`
- [x] `install.sh` one-click + releases GitHub multi-plateformes + .deb + repo APT
- [ ] 2e nœud Windows pair-a-pair complet (binaire dispo, firewall mDNS a ouvrir)
- [ ] Template PyTorch/Jupyter pret-a-fine-tuner

### v0.2 — « 1er pilote externe » (en cours)

- [ ] Parite Windows complete (2e nœud pair-a-pair, firewall mDNS auto)
- [ ] Invitations de bout en bout (join = cles + tunnel auto)
- [ ] Template PyTorch/Jupyter minimal dans la VM
- [ ] **Gate de sortie : 1 pilote externe qui fait 1 transaction** (offre initiale « 3 amis » remplacee : le reseau se construit a partir des solution seekers deja identifies, pas du cercle perso)

### v0.3 — Scale (2027, seulement si gate v0.2 passee)

- [ ] DHT bootstrap auto + persistent identity Ed25519 (WAN)
- [ ] Auto-reconnect + multi-node (3+)
- [ ] Web UI dashboard
- [ ] NCCL multi-GPU, marketplace interne
- [ ] Invitations QR / X / Discord
- [ ] Monitoring & scheduling

---

## MVP Scope

**In scope v0.1**:
- 2 nodes, 1 community, WireGuard handshake, Firecracker 1 template (Ubuntu+PyTorch), SQLite credits, CLI

**Anti-goals v0.1**:
- Pas de GUI, pas de NCCL multi-GPU, pas de blockchain, pas de marketplace public, pas de mobile app, pas de multi-hop

---

## Build Order & Success Criteria

1. WireGuard POC 2 machines — ping OK
2. Discovery DHT — 2 nodes se trouvent
3. Firecracker VM — boot <2s, SSH OK
4. Tunnel+VM — Jupyter `http://10.200.200.2:8888` via tunnel
5. Credits — balance cohérente
6. CLI + install — `<5min` sur Ubuntu 22.04
7. E2E demo — 2 transactions loggees

---

## Quick Start (v0.1, a venir)

```bash
curl -fsSL https://helium.network/install.sh | sh
helium community create --name ma-communaute
helium community offer --ram 32 --cores 8
helium community borrow --ram 16
helium community status
```

Requirements: Linux 5.10+, KVM, 8GB RAM (provider)

---

## Weekly Milestones (4 weeks + buffer)

- **S1 (25-31 Aug)**: Tunnel + Discovery — ping P2P OK
- **S2 (01-07 Sep)**: Firecracker — VM via tunnel OK
- **S3 (08-14 Sep)**: Matching/Credits — 1 transaction OK
- **S4 (15-22 Sep)**: Demos + hardening — MVP complet, gate 25%

Buffer 10-15% inclus.

---

## Progress Mapping

- 25% — Tunnel/Discovery
- 50% — VM integration
- 75% — Matching/Credits
- 95% — E2E + demos

---

## Contributing

Open source MIT a partir de v0.1. Issues et PRs bienvenus apres MVP. Validation par usage (stars/forks) pas par interviews.

---

**Maintained by**: Helium Team — https://github.com/HeliumXChain/Helium
