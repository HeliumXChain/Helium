//! NAT Traversal utilities
//!
//! Implements hole punching techniques for P2P connections
//! through NATs and firewalls.

use anyhow::Result;
use std::net::SocketAddr;

/// NAT traversal coordinator
pub struct NatTraversal {
    stun_servers: Vec<String>,
    local_addr: SocketAddr,
}

impl NatTraversal {
    pub fn new(local_addr: SocketAddr) -> Self {
        Self {
            stun_servers: vec![
                "stun.l.google.com:19302".to_string(),
                "stun1.l.google.com:19302".to_string(),
            ],
            local_addr,
        }
    }

    /// Discover public endpoint via STUN
    pub async fn discover_public_endpoint(&self) -> Result<SocketAddr> {
        tracing::info!("Discovering public endpoint via STUN");
        // TODO: Implement STUN request
        // For now, return local (will fail for NATed peers)
        Ok(self.local_addr)
    }

    /// Perform hole punching to connect to peer
    pub async fn hole_punch(&self, peer_public_addr: SocketAddr) -> Result<SocketAddr> {
        tracing::info!("Attempting hole punch to {}", peer_public_addr);

        // Simultaneous open technique
        // Both peers try to connect to each other at the same time
        // This opens the NAT mapping on both sides

        // TODO: Implement actual hole punching with coordinated timing

        Ok(peer_public_addr)
    }

    /// Check if we're behind NAT
    pub async fn is_behind_nat(&self) -> Result<bool> {
        let public = self.discover_public_endpoint().await?;
        let local = self.local_addr;

        Ok(public.ip() != local.ip())
    }
}

/// Relay server fallback (TURN-like)
pub struct RelayClient {
    relay_addr: SocketAddr,
}

impl RelayClient {
    pub fn new(relay_addr: SocketAddr) -> Self {
        Self { relay_addr }
    }

    /// Connect via relay when P2P fails
    pub async fn connect_via_relay(&self, peer_id: &str) -> Result<RelayConnection> {
        tracing::info!("Connecting via relay to {}", peer_id);
        // TODO: Implement relay protocol
        Ok(RelayConnection {
            peer_id: peer_id.to_string(),
            relay_addr: self.relay_addr,
        })
    }
}

pub struct RelayConnection {
    peer_id: String,
    relay_addr: SocketAddr,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_nat_traversal() {
        let local = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 51820);
        let nat = NatTraversal::new(local);
        assert_eq!(nat.local_addr, local);
    }
}
