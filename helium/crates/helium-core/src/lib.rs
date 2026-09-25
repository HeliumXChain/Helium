//! Helium Core - P2P Discovery and Resource Matching
//!
//! Provides libp2p-based peer discovery, resource advertisement,
//! and matching logic for the Helium compute marketplace.

pub mod discovery;
pub mod matching;
pub mod types;

pub use types::{PeerId, ResourceOffer, ResourceRequest, Match};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Core node managing P2P discovery and resource matching
pub struct HeliumNode {
    peer_id: PeerId,
    discovery: discovery::DiscoveryService,
    matcher: matching::Matcher,
    local_resources: Arc<RwLock<Vec<ResourceOffer>>>,
}

impl HeliumNode {
    pub async fn new() -> anyhow::Result<Self> {
        let peer_id = PeerId::generate();
        let discovery = discovery::DiscoveryService::new(peer_id.clone()).await?;
        let matcher = matching::Matcher::new();

        Ok(Self {
            peer_id,
            discovery,
            matcher,
            local_resources: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Start the node - begins discovery and matching
    pub async fn start(&mut self) -> anyhow::Result<()> {
        tracing::info!("Starting Helium node: {}", self.peer_id);
        self.discovery.start().await?;
        Ok(())
    }

    /// Advertise local resources to the network
    pub async fn advertise_resources(&self, resources: Vec<ResourceOffer>) -> anyhow::Result<()> {
        let mut local = self.local_resources.write().await;
        *local = resources;
        self.discovery.advertise(local.clone()).await?;
        Ok(())
    }

    /// Find matching providers for a resource request
    pub async fn find_matches(&self, request: ResourceRequest) -> anyhow::Result<Vec<Match>> {
        let peers = self.discovery.discover_peers().await?;
        let matches = self.matcher.find_matches(request, peers).await?;
        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_creation() {
        let node = HeliumNode::new().await;
        assert!(node.is_ok());
    }
}
