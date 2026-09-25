# Helium Communities — Architecture Technique

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Helium Communities Stack                         │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                     Application Layer                              │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐ │  │
│  │  │   CLI      │  │  Web UI    │  │  API REST  │  │  SDK Python│ │  │
│  │  │  (Rust)    │  │  (React)   │  │  (Axum)    │  │  (PyO3)    │ │  │
│  │  └──────┬─────┘  └──────┬─────┘  └──────┬─────┘  └──────┬─────┘ │  │
│  │         └─────────────────┴─────────────────┴─────────────┘      │  │
│  └─────────────────────────────┬───────────────────────────────────┘  │
│                                │                                       │
│  ┌─────────────────────────────┼───────────────────────────────────┐  │
│  │                    Core Services Layer                           │  │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │  │
│  │  │  Discovery   │ │   Matching   │ │  Governance  │            │  │
│  │  │  Service     │ │   Engine     │ │   Service    │            │  │
│  │  │  (libp2p)    │ │  (SQLite)    │ │  (Rules)     │            │  │
│  │  └──────────────┘ └──────────────┘ └──────────────┘            │  │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │  │
│  │  │   Tunnel     │ │  Execution   │ │   Token      │            │  │
│  │  │   Service    │ │   Service    │ │   Service    │            │  │
│  │  │ (WireGuard)  │ │(Firecracker) │ │  (Local)     │            │  │
│  │  └──────────────┘ └──────────────┘ └──────────────┘            │  │
│  └─────────────────────────────┬───────────────────────────────────┘  │
│                                │                                       │
│  ┌─────────────────────────────┼───────────────────────────────────┐  │
│  │                    Network Layer                                 │  │
│  │  ┌──────────────────────────────────────────────────────────┐  │  │
│  │  │              P2P Network (libp2p)                          │  │  │
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │  │  │
│  │  │  │Transport │  │  DHT     │  │ Gossip   │  │  NAT     │   │  │  │
│  │  │  │ (QUIC)   │  │(Kademlia)│  │(PubSub)  │  │Traversal │   │  │  │
│  │  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │  │  │
│  │  └──────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

---

## 1. P2P Discovery Layer

### 1.1 Architecture DHT Privée

Chaque communauté opère sa propre DHT isolée via **libp2p** avec configuration personnalisée :

```rust
// helium/communities/src/discovery/mod.rs

use libp2p::{
    kad::{Config as KademliaConfig, Kademlia, store::MemoryStore},
    Multiaddr, PeerId,
};

pub struct CommunityDiscovery {
    community_id: CommunityId,
    kademlia: Kademlia<MemoryStore>,
    bootstrap_nodes: Vec<Multiaddr>,
}

impl CommunityDiscovery {
    pub fn new(
        community_id: CommunityId,
        bootstrap_nodes: Vec<Multiaddr>,
    ) -> Result<Self, DiscoveryError> {
        // Configuration DHT isolée par communauté
        let mut kademlia_config = KademliaConfig::default();

        // Protocol name unique par communauté (isolation complète)
        let protocol_name = format!("/helium/community/{}/kad/1.0.0", community_id);
        kademlia_config.set_protocol_names(vec![protocol_name.into_bytes().into()]);

        // Bucket size, timeout, etc.
        kademlia_config.set_kbucket_inserts(libp2p::kad::BucketInserts::Manual);

        let store = MemoryStore::new(local_peer_id);
        let kademlia = Kademlia::with_config(local_peer_id, store, kademlia_config);

        Ok(Self {
            community_id,
            kademlia,
            bootstrap_nodes,
        })
    }

    /// Annonce les ressources disponibles sur la DHT
    pub fn advertise_resources(&mut self, offer: ResourceOffer) -> Result<(), DiscoveryError> {
        let key = format!("resources/{}/{}", self.community_id, self.local_peer_id);
        let value = serde_json::to_vec(&offer)?;

        self.kademlia.put_record(
            libp2p::kad::Record::new(key.as_bytes(), value),
            libp2p::kad::Quorum::One,
        )?;

        Ok(())
    }

    /// Recherche des ressources dans la communauté
    pub async fn find_resources(
        &mut self,
        requirements: ResourceRequirements,
    ) -> Vec<ResourceOffer> {
        // Query DHT pour tous les records de ressources
        // Filtrer selon les critères (RAM, GPU, disponibilité)
        // Retourner liste triée par reputation_score
        unimplemented!()
    }
}
```

### 1.2 Bootstrap Nodes

Chaque communauté configure ses propres bootstrap nodes (peut être 1+ membres stables) :

```rust
// Configuration bootstrap
pub struct BootstrapConfig {
    /// Nodes statiques pour rejoindre le réseau
    pub static_nodes: Vec<Multiaddr>,
    /// DNS pour résolution dynamique (optionnel)
    pub dns_bootstrap: Option<String>,
    /// Rendezvous server dédié (optionnel)
    pub rendezvous_server: Option<Multiaddr>,
}

// Exemple de configuration "L'Ordre"
let bootstrap = BootstrapConfig {
    static_nodes: vec![
        "/dns4/bootstrap-lome.lordre.local/tcp/4001/p2p/12D3...",
        "/dns4/bootstrap-dakar.lordre.local/tcp/4001/p2p/12D3...",
    ],
    dns_bootstrap: Some("_helium._tcp.lordre.local".to_string()),
    rendezvous_server: None, // Optionnel pour petites communautés
};
```

---

## 2. Secure Tunnel Layer

### 2.1 WireGuard Integration

**Pourquoi WireGuard** :
- Performant (kernel-space sur Linux)
- Simple configuration (clés publiques/privées)
- NAT traversal intégré
- Codebase minimal (4K lignes vs 100K+ pour OpenVPN/IPsec)

```rust
// helium/communities/src/tunnel/wireguard.rs

use wireguard_rs::{Device, DeviceConfig, PeerConfig, KeyPair};

pub struct TunnelManager {
    device: Device,
    local_keypair: KeyPair,
}

impl TunnelManager {
    /// Crée un tunnel vers un provider
    pub async fn create_tunnel(
        &self,
        provider_pubkey: &str,
        provider_endpoint: SocketAddr,
    ) -> Result<Tunnel, TunnelError> {
        let peer_config = PeerConfig {
            public_key: provider_pubkey.parse()?,\n            allowed_ips: vec!["10.200.200.2/32".parse()?], // IP allouée au client
            endpoint: Some(provider_endpoint),
            persistent_keepalive: Some(25), // NAT keepalive
        };

        // Configuration du device WireGuard local
        let device_config = DeviceConfig {
            private_key: self.local_keypair.private.clone(),
            listen_port: 0, // Random port
            addresses: vec!["10.200.200.1/24".parse()?], // IP locale dans le tunnel
            peers: vec![peer_config],
        };

        let device = Device::create("helium-wg0", device_config).await?;

        Ok(Tunnel {
            device,
            local_ip: "10.200.200.1".parse()?,
            remote_ip: "10.200.200.2".parse()?,
        })
    }

    /// Expose un port local via le tunnel
    pub fn expose_port(&self, local_port: u16, remote_port: u16) -> Result<(), TunnelError> {
        // Configure port forwarding dans le tunnel
        // Client local:8080 -> Tunnel -> Provider:8080
        unimplemented!()
    }
}

/// Structure représentant un tunnel actif
pub struct Tunnel {
    device: Device,
    local_ip: IpAddr,
    remote_ip: IpAddr,
}

impl Tunnel {
    /// Envoie du trafic vers le provider via le tunnel
    pub async fn forward(&self, data: &[u8]) -> Result<(), TunnelError> {
        // WireGuard gère automatiquement le routage kernel
        Ok(())
    }

    /// Ferme le tunnel proprement
    pub async fn close(self) -> Result<(), TunnelError> {
        self.device.delete().await?;
        Ok(())
    }
}
```

### 2.2 Alternative : Noise Protocol (Rust Native)

Pour une solution 100% Rust sans dépendance WireGuard :

```rust
// helium/communities/src/tunnel/noise.rs

use snow::{Builder, TransportState};
use tokio::net::TcpStream;

pub struct NoiseTunnel {
    transport: TransportState,
    stream: TcpStream,
}

impl NoiseTunnel {
    /// Pattern Noise : XX (mutual authentication)
    pub fn initiator(private_key: &[u8], remote_pubkey: &[u8]) -> Result<Self, TunnelError> {
        let builder = Builder::new("Noise_XX_25519_ChaChaPoly_BLAKE2s".parse()?);
        let keypair = builder.generate_keypair()?;

        // XX pattern : handshake en 3 messages
        // 1. -> e (ephemeral key)
        // 2. <- e, ee, s, es
        // 3. -> s, se

        unimplemented!()
    }
}
```

### 2.3 NAT Traversal

Pour les nodes derrière NAT (cas le plus courant) :

```rust
// helium/communities/src/tunnel/nat.rs

pub enum NatStrategy {
    /// Hole punching via STUN
    Stun { servers: Vec<String> },
    /// TURN relay (fallback si hole punching échoue)
    Turn { server: String, credentials: TurnCredentials },
    /// P2P direct (si ports ouverts)
    Direct,
}

pub struct NatTraversal {
    strategy: NatStrategy,
    local_addr: SocketAddr,
}

impl NatTraversal {
    /// Tente le hole punching pour établir connexion directe
    pub async fn hole_punch(&self, remote_addr: SocketAddr) -> Result<SocketAddr, NatError> {
        // 1. Les deux côtés envoient des paquets UDP simultanément
        // 2. Le NAT extérieur crée une mapping temporaire
        // 3. Les paquets passent à travers

        // Implémentation avec libp2p-autonat + STUN
        unimplemented!()
    }
}
```

---

## 3. Execution Layer — Firecracker MicroVMs

### 3.1 Firecracker Integration

**Pourquoi Firecracker** :
- MicroVMs légères (< 5MB memory overhead)
- Boot rapide (< 125ms)
- Isolation KVM (sécurité hardware)
- Utilisé par AWS Lambda (battle-tested)

```rust
// helium/communities/src/execution/firecracker.rs

use firecracker_client::models::{InstanceInfo, InstanceActionInfo, Drive, BootSource};

pub struct MicroVM {
    config: VMConfig,
    process: FirecrackerProcess,
}

pub struct VMConfig {
    /// Mémoire allouée (MB)
    pub memory_mb: u64,
    /// vCPUs
    pub vcpus: u32,
    /// Root filesystem (contient le runtime ML)
    pub rootfs_path: PathBuf,
    /// Kernel
    pub kernel_path: PathBuf,
    /// Configuration réseau (via le tunnel)
    pub network_ns: String,
}

impl MicroVM {
    /// Crée une microVM pour exécuter le workload du borrower
    pub async fn create(config: VMConfig) -> Result<Self, ExecutionError> {
        // 1. Lancer processus firecracker
        let socket_path = format!("/tmp/firecracker-{}.sock", uuid::Uuid::new_v4());
        let process = FirecrackerProcess::start(&socket_path).await?;

        // 2. Configurer l'instance
        let instance_config = InstanceInfo {
            vcpu_count: config.vcpus,
            mem_size_mib: config.memory_mb,
            ..Default::default()
        };
        process.configure_machine(instance_config).await?;

        // 3. Configurer le boot source (kernel)
        let boot_source = BootSource {
            kernel_image_path: config.kernel_path.to_string_lossy().to_string(),
            boot_args: Some("console=ttyS0 reboot=k panic=1 pci=off".to_string()),
            ..Default::default()
        };
        process.configure_boot_source(boot_source).await?;

        // 4. Configurer le rootfs
        let root_drive = Drive {
            drive_id: "rootfs".to_string(),
            path_on_host: config.rootfs_path.to_string_lossy().to_string(),
            is_root_device: true,
            is_read_only: true,
            ..Default::default()
        };
        process.configure_drive(root_drive).await?;

        // 5. Configurer le réseau (connecté au tunnel)
        // ...

        // 6. Démarrer l'instance
        process.start_instance().await?;

        Ok(Self { config, process })
    }

    /// Expose un service (ex: Jupyter) via le tunnel
    pub async fn expose_service(&self, port: u16) -> Result<String, ExecutionError> {
        // Retourne l'URL accessible via le tunnel
        // ex: http://10.200.200.2:8888
        unimplemented!()
    }

    /// Arrête et nettoie la VM
    pub async fn destroy(self) -> Result<(), ExecutionError> {
        self.process.shutdown().await?;
        Ok(())
    }
}
```

### 3.2 RootFS Prebuilt

Images rootfs optimisées pour ML inference/training :

```dockerfile
# Dockerfile pour créer rootfs Firecracker
FROM python:3.11-slim

RUN pip install torch transformers accelerate jupyter
RUN pip install vllm text-generation-inference

# Configuration Jupyter
RUN jupyter notebook --generate-config
RUN echo "c.NotebookApp.ip = '0.0.0.0'" >> /root/.jupyter/jupyter_notebook_config.py

# Point d'entrée par défaut
CMD ["jupyter", "notebook", "--no-browser", "--allow-root", "--port=8888"]
```

Conversion en rootfs ext4 pour Firecracker :

```bash
# Créer image ext4
dd if=/dev/zero of=ml-runtime.ext4 bs=1M count=1024
mkfs.ext4 ml-runtime.ext4

# Monter et copier le contenu Docker
cd /mnt && docker export $(docker create ml-runtime) | tar xf -
```

---

## 4. Tokenomics Local

### 4.1 Design Simple (Phase 1)

Système de crédit local sans blockchain :

```rust
// helium/communities/src/token/local.rs

use rusqlite::Connection;

pub struct LocalTokenomics {
    db: Connection,
    community_id: CommunityId,
}

/// Un "crédit" représente un droit d'usage futur
#[derive(Debug, Clone)]
pub struct ComputeCredit {
    pub owner: NodeId,
    pub resource_type: ResourceType, // RamHour, GpuHour, etc.
    pub quantity: u64,
    pub expires_at: Option<DateTime<Utc>>,
    pub issued_by: NodeId, // Qui a émis ce crédit (le prêteur)
}

impl LocalTokenomics {
    /// Emprunteur reçoit des crédits en échange d'un prêt
    pub fn issue_credit(
        &self,
        borrower: NodeId,
        lender: NodeId,
        resource_type: ResourceType,
        quantity: u64,
    ) -> Result<ComputeCredit, TokenError> {
        // Crédit = promesse d'accès futur
        // Ex: "Tu m'as prêté 1h GPU, je te dois 1h GPU"

        let credit = ComputeCredit {
            owner: lender, // Le prêteur possède le crédit
            resource_type,
            quantity,
            expires_at: Some(Utc::now() + Duration::days(30)), // Expire dans 30j
            issued_by: borrower,
        };

        // Persist dans SQLite local
        self.db.execute(
            "INSERT INTO credits (owner, issuer, resource_type, quantity, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (&credit.owner, &credit.issued_by, &credit.resource_type,
             &credit.quantity, &credit.expires_at),
        )?;

        Ok(credit)
    }

    /// Consomme un crédit pour accéder à des ressources
    pub fn consume_credit(
        &self,
        owner: NodeId,
        resource_type: ResourceType,
        quantity: u64,
    ) -> Result<(), TokenError> {
        // Vérifie que owner a suffisamment de crédits
        // Décrémente le solde
        // Enregistre la transaction
        unimplemented!()
    }

    /// Solde de crédits d'un node
    pub fn get_balance(&self, node: NodeId) -> Result<Balance, TokenError> {
        // SELECT SUM(quantity) FROM credits WHERE owner = ? AND expires_at > now
        unimplemented!()
    }
}
```

### 4.2 Modèles de Crédit

| Modèle | Description | Cas d'usage |
|--------|-------------|-------------|
| **Direct** | 1h prêtée = 1h reçue immédiatement | Sessions courtes, besoin immédiat |
| **Différé** | 1h prêtée = 1h crédit valable 30j | Planification, flexibilité |
| **Taux variable** | RAM prêtée peut être échangée contre GPU | Hétérogénéité des ressources |
| **Don** | Prêt sans contrepartie (mentoring) | Seniors aidant juniors |

---

## 5. Data Flow — Séquence Complète

```
Borrower (Lomé)                    Provider (Dakar)
     │                                    │
     │  1. discover(resources)            │
     │ ─────────────────────────────────> │
     │      (Query DHT locale)            │
     │                                    │
     │  2. ResourceOffer (32GB, RTX 4080) │
     │ <───────────────────────────────── │
     │                                    │
     │  3. request_borrow(offer_id)       │
     │ ─────────────────────────────────> │
     │      (Négociation, conditions)     │
     │                                    │
     │  4. approve + pubkey + endpoint    │
     │ <───────────────────────────────── │
     │                                    │
     │  5. create_tunnel(pubkey)          │
     │ ─────────────────────────────────> │
     │      (WireGuard handshake)         │
     │                                    │
     │  6. Tunnel established             │
     │ <══════════════════════════════════> │
     │      (10.200.200.0/24)             │
     │                                    │
     │  7. create_firecracker_vm(config)  │
     │ ─────────────────────────────────> │
     │      (VM 16GB RAM, 4 cores)        │
     │                                    │
     │  8. VM ready + Jupyter URL         │
     │ <───────────────────────────────── │
     │                                    │
     │  9. Access Jupyter via tunnel      │
     │      http://10.200.200.2:8888     │
     │═══════════════════════════════════>│
     │                                    │
     │  10. Execute ML workload           │
     │      (fine-tuning LLM)             │
     │═══════════════════════════════════>│
     │                                    │
     │  11. stop_borrow()                 │
     │ ─────────────────────────────────> │
     │                                    │
     │  12. destroy_vm() + close_tunnel │
     │      issue_credit(borrower, 1h)    │
     │ <───────────────────────────────── │
     │                                    │
```

---

## 6. Security Model

### 6.1 Isolation Multi-Couches

```
┌─────────────────────────────────────────────────────────┐
│                    Layer 1: Network                      │
│  • Tunnel chiffré (WireGuard/Noise)                     │
│  • Pas d'accès réseau externe depuis la VM              │
│  • Firewall stateful                                    │
├─────────────────────────────────────────────────────────┤
│                    Layer 2: VM                           │
│  • Firecracker microVM (KVM isolation)                    │
│  • Rootfs read-only                                     │
│  • Pas de device block supplémentaire                  │
├─────────────────────────────────────────────────────────┤
│                    Layer 3: Resource                     │
│  • Cgroups pour limiter CPU/RAM                        │
│  • Timeout hard kill après durée max                   │
│  • Pas d'accès GPU si pas demandé                      │
├─────────────────────────────────────────────────────────┤
│                    Layer 4: Trust                        │
│  • Reputation score par communauté                       │
│  • Web of Trust (invitation membre)                      │
│  • Slashing pour comportement abusif                   │
└─────────────────────────────────────────────────────────┘
```

### 6.2 Gestion des Risques

| Risque | Mitigation | Implémentation |
|--------|------------|----------------|
| Code malveillant | Sandbox + image read-only | Firecracker + rootfs verrouillé |
| Exfiltration data | No outbound network | Firewall DROP all except tunnel |
| DoS ressources | Quotas + timeouts | Cgroups + timer watchdog |
| Node malveillant | Reputation + ban | SQLite reputation score |
| Sybil attack | Web of Trust | Invitation par membres existants |
| Eavesdropping | Chiffrement | WireGuard ChaCha20-Poly1305 |

---

## 7. MVP Scope (Phase 1)

### 7.1 Must Have (Core)

- [ ] DHT privée par communauté (libp2p)
- [ ] Discovery ressources basique
- [ ] Tunnel WireGuard fonctionnel
- [ ] Firecracker VM simple
- [ ] Tokenomics crédit local (SQLite)
- [ ] CLI basique (create, join, offer, borrow)

### 7.2 Should Have (UX)

- [ ] Web UI pour visualiser ressources
- [ ] Rootfs prébuildés (PyTorch, Jupyter)
- [ ] NAT traversal (STUN/hole punching)
- [ ] Reputation score basique
- [ ] Logs et monitoring

### 7.3 Nice to Have (Future)

- [ ] Integration tokens Helium publics
- [ ] Proof of Community Work (PoCW)
- [ ] Gouvernance on-chain communautaire
- [ ] Multi-hop routing (borrower -> A -> B -> provider)
- [ ] Auto-scaling (demander ressources à multiple providers)

---

## 8. Dependencies

### Rust Crates

```toml
[dependencies]
# P2P Networking
libp2p = { version = "0.53", features = ["kad", "noise", "quic", "tokio"] }
libp2p-quic = "0.10"

# Cryptography
snow = "0.9"           # Noise Protocol
wireguard-rs = "0.4"   # WireGuard (optional)
ring = "0.17"          # Crypto primitives

# VM Management
firecracker-client = "1.5"  # API client Firecracker
firecracker-microvm = "1.5" # MicroVM management

# Networking
tokio = { version = "1.35", features = ["full"] }
quinn = "0.10"         # QUIC (alternative)

# Database
rusqlite = { version = "0.30", features = ["bundled", "chrono"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# CLI
clap = { version = "4.4", features = ["derive"] }
table = "0.10"         # Pretty tables for CLI output

# Async
async-trait = "0.1"
futures = "0.3"

# Utilities
uuid = { version = "1.6", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
tracing = "0.1"
```

---

## 9. Testing Strategy

### 9.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_advertisement() {
        let mut discovery = CommunityDiscovery::new(
            CommunityId::new("test-community"),
            vec![], // Pas de bootstrap pour test
        ).unwrap();

        let offer = ResourceOffer {
            node_id: NodeId::random(),
            resources: ResourceCapacity {
                ram_gb: 32,
                cpu_cores: 8,
                gpu: None,
                ..Default::default()
            },
            pricing: PricingModel::CreditPerHour(1),
            availability: AvailabilityWindow::Always,
            reputation_score: 100,
        };

        discovery.advertise_resources(offer).unwrap();

        // Vérifier que l'offre est dans la DHT locale
        let found = discovery.find_resources(ResourceRequirements {
            min_ram_gb: 16,
            ..Default::default()
        }).await;

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].resources.ram_gb, 32);
    }

    #[tokio::test]
    async fn test_tunnel_creation() {
        // Test tunnel WireGuard entre deux pairs virtuels
        // Vérifier le handshake et l'échange de clés
    }
}
```

### 9.2 Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_borrow() {
    // Setup: 2 nodes dans une communauté de test
    // Node A: Provider avec 16GB RAM
    // Node B: Borrower qui demande 8GB

    // 1. Node A advertise resources
    // 2. Node B discover and request borrow
    // 3. Establish tunnel
    // 4. Create Firecracker VM
    // 5. Verify VM accessible via tunnel
    // 6. Stop borrow and cleanup
    // 7. Verify credit issued
}
```

---

## 10. Deployment

### 10.1 Requirements Système

| Composant | Minimum | Recommandé |
|-----------|---------|------------|
| **Provider (prêteur)** | 8GB RAM, 4 cores | 32GB RAM, 8+ cores, GPU |
| **Borrower (emprunteur)** | 2GB RAM, 2 cores | 4GB RAM, 4 cores |
| **OS** | Linux kernel 5.10+ | Ubuntu 22.04 LTS |
| **Kernel features** | KVM, WireGuard | KVM, WireGuard, cgroup v2 |

### 10.2 Installation

```bash
# Installer Helium Communities
curl -fsSL https://helium.network/install-communities.sh | sh

# Configurer comme provider (machine avec ressources)
helium-community configure --mode provider --ram 32 --cores 8

# Configurer comme borrower (machine légère)
helium-community configure --mode borrower

# Rejoindre une communauté
helium-community join --community-id "lordre-west-africa" --invite-code "XYZ123"

# Démarrer le service
sudo systemctl enable helium-community
sudo systemctl start helium-community
```
