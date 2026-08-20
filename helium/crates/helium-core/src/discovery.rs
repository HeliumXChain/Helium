//! P2P Discovery Service using libp2p
//!
//! Handles peer discovery via mDNS (local network)
//! Protocol Helium: échange d'identités et de ressources entre peers

use crate::{types::{PeerInfo, ResourceOffer, ResourceType}, PeerId};
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::info;

/// Service de découverte P2P
pub struct DiscoveryService {
    local_id: PeerId,
    known_peers: RwLock<HashMap<String, PeerInfo>>,
    advertised_resources: RwLock<Vec<ResourceOffer>>,
    discovery_active: RwLock<bool>,
}

impl DiscoveryService {
    pub async fn new(local_id: PeerId) -> Result<Self> {
        Ok(Self {
            local_id,
            known_peers: RwLock::new(HashMap::new()),
            advertised_resources: RwLock::new(Vec::new()),
            discovery_active: RwLock::new(false),
        })
    }

    /// Démarre le service de découverte
    pub async fn start(&mut self) -> Result<()> {
        info!("Démarrage du service de découverte pour {}", self.local_id);

        let mut active = self.discovery_active.write().await;
        *active = true;

        // TODO: Intégration libp2p avec mDNS
        // Pour l'instant, le service fonctionne en mode "manuel"
        // où les peers sont ajoutés via add_peer()

        info!("Service de découverte actif (mode manuel)");
        Ok(())
    }

    /// Publie les ressources locales sur le réseau
    pub async fn advertise(&self, resources: Vec<ResourceOffer>) -> Result<()> {
        let mut advertised = self.advertised_resources.write().await;
        *advertised = resources;
        info!("Publication de {} ressources sur le réseau", advertised.len());

        // TODO: Publier dans la DHT Kademlia
        // Pour mDNS, les peers seront découverts automatiquement

        Ok(())
    }

    /// Découvre les peers disponibles sur le réseau
    pub async fn discover_peers(&self) -> Result<Vec<PeerInfo>> {
        let peers = self.known_peers.read().await;
        let peer_list: Vec<PeerInfo> = peers.values().cloned().collect();
        info!("{} peers connus", peer_list.len());
        Ok(peer_list)
    }

    /// Ajoute un peer découvert manuellement ou via mDNS
    pub async fn add_peer(&self, peer: PeerInfo) {
        let mut peers = self.known_peers.write().await;
        let peer_id_str = peer.peer_id.as_str().to_string();

        if !peers.contains_key(&peer_id_str) {
            info!("Nouveau peer découvert: {} avec {} ressources",
                peer_id_str, peer.resources.len());
            peers.insert(peer_id_str, peer);
        }
    }

    /// Recherche des peers avec des ressources spécifiques
    pub async fn find_peers_with_resources(&self, resource_type: ResourceType) -> Result<Vec<PeerInfo>> {
        let peers = self.known_peers.read().await;
        let matching_peers: Vec<PeerInfo> = peers
            .values()
            .filter(|p| p.resources.iter().any(|r| r.resource_type == resource_type))
            .cloned()
            .collect();

        info!("{} peers avec ressources '{:?}'", matching_peers.len(), resource_type);
        drop(peers); // Explicit drop to satisfy clippy

        Ok(matching_peers)
    }

    /// Obtient les ressources publiées par un peer spécifique
    pub async fn get_peer_resources(&self, peer_id: &str) -> Option<Vec<ResourceOffer>> {
        let peers = self.known_peers.read().await;
        peers.get(peer_id).map(|p| p.resources.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ResourceType, ResourceCapacity, AvailabilityWindow};

    fn create_test_availability() -> AvailabilityWindow {
        AvailabilityWindow {
            start_time: chrono::Utc::now(),
            duration_hours: 24,
        }
    }

    #[tokio::test]
    async fn test_discovery_service_creation() {
        let id = PeerId::generate();
        let service = DiscoveryService::new(id).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_add_and_discover_peer() {
        let id = PeerId::generate();
        let service = DiscoveryService::new(id).await.unwrap();

        let peer_id = PeerId::generate();
        let peer = PeerInfo {
            peer_id: peer_id.clone(),
            endpoint: "127.0.0.1:8080".to_string(),
            resources: vec![ResourceOffer {
                peer_id: peer_id.clone(),
                resource_type: ResourceType::Gpu,
                capacity: ResourceCapacity {
                    amount: 1,
                    unit: "GPU".to_string(),
                    gpu_model: Some("RTX 4090".to_string()),
                },
                price_per_hour: 2.0,
                availability: create_test_availability(),
            }],
            last_seen: chrono::Utc::now(),
        };

        service.add_peer(peer).await;

        let peers = service.discover_peers().await.unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].peer_id, peer_id);
    }

    #[tokio::test]
    async fn test_find_peers_with_gpu() {
        let id = PeerId::generate();
        let service = DiscoveryService::new(id).await.unwrap();

        // Ajouter un peer avec GPU
        let peer_with_gpu = PeerInfo {
            peer_id: PeerId::generate(),
            endpoint: "127.0.0.1:8081".to_string(),
            resources: vec![ResourceOffer {
                peer_id: PeerId::generate(),
                resource_type: ResourceType::Gpu,
                capacity: ResourceCapacity {
                    amount: 1,
                    unit: "GPU".to_string(),
                    gpu_model: Some("RTX 4090".to_string()),
                },
                price_per_hour: 2.0,
                availability: create_test_availability(),
            }],
            last_seen: chrono::Utc::now(),
        };

        // Ajouter un peer avec CPU seulement
        let peer_with_cpu = PeerInfo {
            peer_id: PeerId::generate(),
            endpoint: "127.0.0.1:8082".to_string(),
            resources: vec![ResourceOffer {
                peer_id: PeerId::generate(),
                resource_type: ResourceType::Cpu,
                capacity: ResourceCapacity {
                    amount: 16,
                    unit: "cores".to_string(),
                    gpu_model: None,
                },
                price_per_hour: 1.0,
                availability: create_test_availability(),
            }],
            last_seen: chrono::Utc::now(),
        };

        service.add_peer(peer_with_gpu).await;
        service.add_peer(peer_with_cpu).await;

        let gpu_peers = service.find_peers_with_resources(ResourceType::Gpu).await.unwrap();
        assert_eq!(gpu_peers.len(), 1);
    }
}
