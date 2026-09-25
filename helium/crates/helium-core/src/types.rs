//! Core types for Helium resource marketplace

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a peer
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(String);

impl PeerId {
    pub fn generate() -> Self {
        // TODO: Use proper keypair generation from libp2p
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0[..8])
    }
}

/// Compute resource types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Cpu,
    Ram,
    Gpu,
}

/// Resource capabilities offered by a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceOffer {
    pub peer_id: PeerId,
    pub resource_type: ResourceType,
    pub capacity: ResourceCapacity,
    pub price_per_hour: f64,
    pub availability: AvailabilityWindow,
}

/// Resource capacity specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapacity {
    /// For CPU: number of cores
    /// For RAM: GB
    /// For GPU: model + VRAM GB
    pub amount: u32,
    pub unit: String,
    pub gpu_model: Option<String>, // Only for GPU
}

/// When the resource is available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityWindow {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub duration_hours: u32,
}

/// Resource request from a borrower
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequest {
    pub requester_id: PeerId,
    pub resource_type: ResourceType,
    pub min_capacity: ResourceCapacity,
    pub max_price_per_hour: f64,
    pub needed_duration_hours: u32,
    pub deadline: chrono::DateTime<chrono::Utc>,
}

/// A match between request and offer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub offer: ResourceOffer,
    pub request: ResourceRequest,
    pub score: f64, // 0.0 - 1.0 match quality
    pub estimated_cost: f64,
}

/// Peer information discovered on the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub endpoint: String, // IP:port
    pub resources: Vec<ResourceOffer>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}
