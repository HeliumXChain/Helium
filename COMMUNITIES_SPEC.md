# Helium Communities — Specification

## Overview

Extension de Helium permettant la création de **réseaux P2P privés** où les membres se prêtent mutuellement des ressources compute (RAM/GPU) pour exécuter des modèles AI locaux.

**Différence avec Helium Core** :
- Helium Core = Marketplace global, Proof of Useful Work, tokens publics
- Helium Communities = Réseaux fermés, confiance sociale, prêt contre accès futur

---

## Use Case Principal

**Scénario** : Un développeur à Lomé a besoin de 32GB RAM pour fine-tuner un modèle LLM. Son laptop n'a que 16GB. Un autre membre de sa communauté a un PC workstation avec 64GB allumé 24/7 mais inutilisé 80% du temps.

**Solution** : Via Helium Communities, le développeur :
1. Découvre le membre avec ressources disponibles (réseau privé)
2. Négocie un prêt de 16GB RAM pour 2 heures
3. Établit un tunnel chiffré WireGuard/Noise entre les deux machines
4. Exécute son fine-tuning via le tunnel
5. Paie en tokens communautaires ou crédit d'accès futur

---

## Architecture Technique

### 1. Discovery Layer — Réseaux Privés

```
┌─────────────────────────────────────────────────────────┐
│                  Helium Communities                      │
│                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌────────────┐ │
│  │ Community A  │    │ Community B  │    │ Community C │ │
│  │ (L'Ordre)    │    │ (DevNairobi) │    │ (MLAccra)   │ │
│  │              │    │              │    │             │ │
│  │ • Node Lomé  │    │ • Node NBO-1 │    │ • Node ACC-1│ │
│  │ • Node Dakar │    │ • Node NBO-2 │    │ • Node ACC-2│ │
│  │ • Node Abidjan│   │ • Node NBO-3 │    │ • Node ACC-3│ │
│  └──────────────┘    └──────────────┘    └────────────┘ │
│                                                          │
│  Chaque communauté = sous-réseau P2P isolé              │
│  Discovery via DHT privé ou bootstrap nodes dédiés      │
└─────────────────────────────────────────────────────────┘
```

**Implémentation** :
- Chaque communauté a sa propre DHT (Distributed Hash Table)
- Bootstrap nodes configurables par l'admin de la communauté
- Zero-knowledge de l'existence des autres communautés (privacy)

### 2. Resource Advertisement

Chaque nœud annonce ses ressources disponibles :

```rust
struct ResourceOffer {
    community_id: CommunityId,
    node_id: NodeId,
    resources: ResourceCapacity,
    pricing: PricingModel,
    availability: AvailabilityWindow,
    reputation_score: u32,  // Historique de prêt dans la communauté
}

struct ResourceCapacity {
    cpu_cores: u32,
    ram_gb: u32,
    gpu: Option<GPUInfo>,  // CUDA, ROCm, ou CPU-only
    storage_gb: u32,
    bandwidth_mbps: u32,
}
```

### 3. Secure Tunnel — WireGuard/Noise Protocol

```
┌──────────────┐         WireGuard Tunnel          ┌──────────────┐
│   Client     │ ◄────────────────────────────────► │   Provider   │
│  (Borrower)  │    Chiffrement ChaCha20-Poly1305   │   (Lender)   │
│              │         + P2P NAT Traversal        │              │
│ • Needs GPU  │                                    │ • Has GPU    │
│ • Local IDE  │◄───── Expose port 8080 via tunnel ─►│ • Docker/VM  │
│ • Jupyter    │                                    │ • LLM runtime│
└──────────────┘                                    └──────────────┘
```

**Stack technique** :
- WireGuard pour le tunnel chiffré (performant, léger)
- Noise Protocol comme alternative Rust-native (libp2p-noise)
- NAT traversal via STUN/TURN ou hole punching P2P

### 4. Execution Environment

**Options d'isolation** :

| Niveau | Sécurité | Performance | Use Case |
|--------|----------|-------------|----------|
| Docker container | Moyenne | Excellente | Code trusté, membres connus |
| Firecracker microVM | Haute | Très bonne | Multi-tenant, sandboxing |
| KVM/VM | Maximale | Bonne | Exigence sécurité maximale |

**Recommandation** : Firecracker pour le meilleur ratio sécurité/performance.

### 5. Tokenomics Communautaire

**Modèle de prêt** :

```
┌─────────────────────────────────────────────────────┐
│           Tokenomics Communautaire                  │
│                                                      │
│  Prêt de ressources ──► Reçoit tokens/crédits       │
│                                                      │
│  Modèles possibles :                                │
│  1. Direct : 1h GPU = X tokens (paiement immédiat)  │
│  2. Crédit : Prêt aujourd'hui → Accès futur garanti │
│  3. Troc : 1h RAM donnée = 1h RAM reçue plus tard   │
│  4. Subvention : Seniors prêtent aux juniors (gratuit│
│                                                      │
│  Tokens communautaires ≠ Tokens Helium publics      │
│  • Non échangeables hors communauté                  │
│  • Règles définies par la communauté (gouvernance)  │
│  • Émission contrôlée par smart contract communautaire│
└─────────────────────────────────────────────────────┘
```

---

## Modules Rust Proposés

```
helium/
├── core/                    # Helium Core (blockchain, PoUW)
├── network/                 # P2P networking global
├── communities/             # NOUVEAU — Extension Communities
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── discovery/       # DHT privée, bootstrap
│   │   ├── tunnel/          # WireGuard/Noise implementation
│   │   ├── resource/        # Advertisement, matching
│   │   ├── execution/       # Docker/Firecracker isolation
│   │   ├── token/           # Tokenomics communautaire
│   │   └── governance/      # Règles de la communauté
│   └── tests/
└── cli/
    └── src/
        └── communities.rs   # Commandes CLI pour gérer communities
```

---

## CLI Interface

```bash
# Créer une communauté
helium community create --name "L'Ordre" --region "West Africa"

# Rejoindre une communauté (via invite code)
helium community join <COMMUNITY_ID> --invite-code <CODE>

# Publier ses ressources disponibles
helium community offer --ram 32 --gpu "RTX 4080" --availability "20:00-08:00"

# Découvrir les ressources disponibles
helium community discover --resource-type ram --min-gb 16

# Emprunter des ressources (établit le tunnel automatiquement)
helium community borrow --from <NODE_ID> --ram 16 --duration 2h --tunnel-port 8080

# Voir son solde de tokens communautaires
helium community balance

# Historique de prêt/emprunt
helium community history --reputation
```

---

## Intégration avec Helium Core

**Option A — Parallèle** (Recommandé pour MVP) :
- Communities fonctionne en standalone
- Pas de dépendance au blockchain Helium
- Tokenomics interne à chaque communauté

**Option B — Intégrée** :
- Wrap/unwrap des tokens Helium publics
- Proof of Community Work (PoCW) pour récompenser les prêteurs
- Bridge entre économie locale et globale

**Recommandation Phase 1** : Option A — démarrer simple, sans blockchain.

---

## Security Considerations

| Risque | Mitigation |
|--------|------------|
| Node malveillant | Reputation score + slashing + ban communautaire |
| Data exfiltration | VMs isolées, pas d'accès réseau libre |
| DDoS du provider | Rate limiting, auto-cutoff après durée max |
| Code malveillant | Sandboxing Firecracker + scan antivirus |
| Sybil attack | Invitation par membres existants (Web of Trust) |

---

## Prochaines Étapes

1. [ ] Valider le besoin avec Mom Test adapté (devs africains sans GPU)
2. [ ] POC tunnel WireGuard P2P entre 2 machines
3. [ ] MVP Rust : discovery + matching simple
4. [ ] Intégration Firecracker pour isolation
5. [ ] Tokenomics communautaire (simple crédit d'abord)
