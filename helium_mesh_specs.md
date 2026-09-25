# Helium Mesh — Spécifications Techniques

**Version** : 0.1.0-alpha
**Date** : 2026-04-04
**Status** : Draft

---

## 1. Architecture Overview

### Network Topology
- **Mesh Network** : Full mesh WireGuard VPN between all nodes
- **Auto-Reconnect** : DHT-based peer discovery + persistent keys
- **Web of Trust** : Invitation-based joining, vouching system

### Core Components

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **Networking** | WireGuard | VPN mesh between nodes |
| **Discovery** | DHT (libp2p) | Peer discovery and bootstrap |
| **Isolation** | Firecracker | MicroVM per workload |
| **GPU Pooling** | NCCL over VPN | Multi-GPU cluster |
| **Storage** | IPFS / MinIO | Distributed dataset storage |
| **Identity** | Ed25519 keys | Cryptographic identity |
| **Credit System** | Local SQLite | Accounting between peers |

---

## 2. Technical Stack

### Backend (Node Daemon)
```
Language: Rust (performance, safety)
Key crates:
  - wireguard-rs: VPN tunnel management
  - firecracker-rs: MicroVM orchestration
  - nccl: NVIDIA collective communications
  - libp2p: DHT peer discovery
  - ipfs-embed: Storage layer
  - rusqlite: Local credit database
  - tokio: Async runtime
```

### Frontend (Web UI)
```
Stack: Next.js + Tailwind + shadcn/ui
Features:
  - Node dashboard (status, resources)
  - Invitation generation (links, QR)
  - Workload launcher (Jupyter, SSH, Training)
  - Credit balance and history
  - Web of Trust visualization
```

### CLI Tool
```
Language: Rust (same codebase)
Commands:
  helium init           # Initialize node
  helium mesh join      # Join via invitation
  helium mesh status    # Show mesh topology
  helium workload run   # Launch workload
  helium credits send   # Send credits to peer
```

---

## 3. Data Flows

### 3.1 Node Bootstrap
```
1. User runs: curl https://helium.sh | sh
2. Generate Ed25519 keypair (persistent identity)
3. Generate WireGuard keypair (ephemeral, rotated)
4. Join DHT bootstrap nodes (hardcoded initially)
5. Listen for invitations or create mesh
```

### 3.2 Invitation Flow
```
Alice (existing member):
  1. Generates invitation: helium invite create --peer bob
  2. Creates signed payload: {bob_pubkey, expires, nonce}
  3. Generates short code + full URL
  4. Shares: X DM / WhatsApp / Discord

Bob (new member):
  1. Receives link: https://helium.sh/join#<encrypted_payload>
  2. Runs: helium mesh join --invite <code>
  3. Verifies signature with Alice pubkey
  4. Establishes WireGuard tunnel to Alice
  5. Alice "vouches" → Bob added to Web of Trust
  6. Bob discovers other peers via DHT
```

### 3.3 Workload Execution
```
User requests GPU on Node B from Node A:
  1. Node A sends request: {gpu_count, duration, docker_image}
  2. Node B validates: credit balance, WoT status
  3. Node B spins up Firecracker VM with GPU passthrough
  4. WireGuard tunnel established: Node A <-> Firecracker VM
  5. User connects via SSH/Jupyter/HTTP proxy
  6. Real-time credit deduction (per second)
  7. VM destroyed on completion or credit exhaustion
```

### 3.4 GPU Pooling (Multi-Node Training)
```
4 nodes, 1 GPU each → 1 virtual 4-GPU cluster:
  1. Nodes establish NCCL ring over WireGuard mesh
  2. Each node exposes GPU via CUDA MPS (Multi-Process Service)
  3. PyTorch/TensorFlow sees 4 GPUs via NCCL backend
  4. Training distributed across nodes
  5. Gradient synchronization via NCCL all-reduce
  6. Credit split proportionally to GPU contribution
```

---

## 4. Security Model

### Threats Mitigated

| Threat | Mitigation |
|--------|------------|
| **Stranger attack** | Web of Trust — only invited/vouched peers |
| **VM escape** | Firecracker — KVM isolation, no shared kernel |
| **Network sniffing** | WireGuard — ChaCha20-Poly1305 encryption |
| **Data exfiltration** | Encrypted volumes, no host access |
| **Credit fraud** | Local consensus, signed transactions |
| **DDoS** | Rate limiting, resource quotas |

### Web of Trust Details
```
Graph structure:
  - Nodes = Ed25519 public keys
  - Edges = vouch relationships (signed attestations)
  - Trust score = weighted shortest path from root
  - Threshold = configurable (default: 2-hop trust)

Bootstrap:
  - Initial node = trust anchor
  - New nodes need 1+ vouch from existing trusted node
  - Revocation = signed un-vouch message propagates
```

### Credit System
```
Local SQLite per node:
  - Table: balances {peer_pubkey, credits}
  - Table: transactions {tx_id, from, to, amount, timestamp, signature}

Consensus:
  - Each transaction signed by sender
  - Receiver validates signature before accepting
  - Periodic reconciliation between peers
  - Dispute resolution: manual (MVP), automated (v2)
```

---

## 5. Storage Architecture

### Option A: IPFS (Recommended for MVP)
```
Pros:
  - Content-addressed (deduplication automatic)
  - NAT traversal built-in
  - Mature ecosystem
Cons:
  - Slower for large files (workaround: local cache)
  - Public DHT (privacy: encrypt everything)

Implementation:
  - Dataset uploaded to IPFS by owner
  - Encrypted with recipient's public key
  - Peers pin dataset for duration of training
  - Local badgerds cache for hot data
```

### Option B: MinIO (S3-compatible)
```
Pros:
  - S3 API compatibility
  - Faster for sequential reads
Cons:
  - Requires persistent storage on each node
  - Manual replication management

Implementation:
  - Distributed MinIO cluster across mesh
  - Erasure coding for redundancy
  - Gateway mode to IPFS for cold storage
```

---

## 6. Milestones & Phases

### Phase 1: MVP (2-4 semaines)
**Goal**: 2-3 nodes, basic sharing
- [ ] WireGuard mesh between 2 nodes
- [ ] Firecracker VM with GPU passthrough
- [ ] Simple CLI (init, join, run)
- [ ] Invitation links via copy-paste
- [ ] Credit system (SQLite, simple)
- [ ] 1 workload: SSH into GPU VM

**Success Criteria**:
- 2 transactions réussies
- Setup < 10 minutes
- Satisfaction > 4/5

### Phase 2: Mesh Persistant (2-3 semaines)
**Goal**: Auto-reconnect, pool VRAM
- [ ] DHT peer discovery
- [ ] Persistent identity (Ed25519)
- [ ] Auto-reconnect after reboot
- [ ] NCCL ring formation
- [ ] Multi-GPU training (PyTorch)
- [ ] Web UI (Next.js)

**Success Criteria**:
- 3+ nodes stable
- Training 2-GPU fonctionnel
- Web UI onboarding < 5 min

### Phase 3: Social & Enterprise (3-4 semaines)
**Goal**: Invitations sociaux, stockage
- [ ] Liens d'invitation X/WhatsApp/Discord
- [ ] QR code onboarding
- [ ] IPFS storage integration
- [ ] Web of Trust UI
- [ ] Credit reconciliation
- [ ] Enterprise features (SSO, audit logs)

**Success Criteria**:
- 10+ utilisateurs
- 3 communautés
- 1000+ heures compute mensuel

### Phase 4: Scale (2+ mois)
**Goal**: Production-ready
- [ ] Automatic credit reconciliation
- [ ] GPU marketplace (internal pricing)
- [ ] Advanced scheduling (priority queues)
- [ ] Monitoring & alerting
- [ ] Mobile app (view-only)

---

## 7. Open Source Strategy

### Recommandation: Hybrid Open Core

**Open Source (GitHub Public)**:
```
- Client CLI (Rust)
- Web UI (Next.js)
- Protocol specifications
- Firecracker integration layer
- Basic documentation
```

**Propriétaire / Private**:
```
- Enterprise features (SSO, audit)
- Managed relay service (SaaS option)
- Advanced scheduling algorithms
- Proprietary optimizations
```

### Stratégie de Communication

**Phase MVP (Maintenant)** : **Privé**
- Pas de repo public
- Développement en privé
- Partage uniquement avec co-fondateurs/testeurs

**Raisons** :
1. **First-mover protection** — Éviter que Vast.ai/RunPod copient immédiatement
2. **Validation d'abord** — Valider le problème avant d'ouvrir
3. **Complexité** — Pas besoin de communauté open source pour MVP

**Phase Post-Validation (Mois 2-3)** : **Open Source Client**
- Ouvrir le client CLI et Web UI
- Garder certaines features enterprise privées
- Bénéfices: confiance, contributions, marketing

### Protection IP

| Element | Protection | Action |
|---------|------------|--------|
| **Architecture** | Trade secret | Pas de doc publique pour l'instant |
| **Code** | Copyright | License Apache 2.0 (future) |
| **Brand** | Trademark | Déposer "Helium Mesh" |
| **Inventions** | Patents (optionnel) | Évaluer après validation |

---

## 8. Risques Techniques

| Risque | Probabilité | Impact | Mitigation |
|--------|-------------|--------|------------|
| **NCCL over VPN trop lent** | Moyen | Haut | Fallback: gradient compression, async training |
| **Firecracker GPU passthrough** | Faible | Haut | Alternative: gVisor, ou bare-metal isolation |
| **DHT bootstrap failure** | Moyen | Moyen | Hardcoded bootstrap nodes + manual peer add |
| **IPFS lenteur** | Moyen | Moyen | Local caching layer (badgerds) |
| **Credit system abuse** | Moyen | Moyen | Start with trusted circle only |

---

## 9. Next Steps (Immediate)

**Cette semaine** :
1. [ ] Valider architecture avec desk research
2. [ ] Créer repo privé GitHub
3. [ ] Setup dev environment (Rust, Firecracker)
4. [ ] Prototype WireGuard mesh (2 VMs locales)

**Semaine 2** :
5. [ ] Firecracker + GPU prototype
6. [ ] CLI commands (init, join, run)
7. [ ] Test transaction 1: Alice → Bob

---

**Document Owner** : CEO/CTO Helium
**Distribution** : Privé — Core team only
**Review Cycle** : Weekly pendant MVP
