//! Helium Tunnel - WireGuard P2P Tunnels
//! 
//! Manages secure P2P tunnels between borrowers and providers
//! using WireGuard for encryption and NAT traversal.

pub mod wireguard;
pub mod nat;

use crate::wireguard::{WireGuardConfig, WireGuardInterface};
use anyhow::Result;
use std::net::SocketAddr;

/// Tunnel manager for P2P connections
pub struct TunnelManager {
    local_interface: Option<WireGuardInterface>,
    config: TunnelConfig,
}

#[derive(Debug, Clone)]
pub struct TunnelConfig {
    pub listen_port: u16,
    pub private_key: String,
    pub public_key: String,
    pub allowed_ips: Vec<String>,
}

impl TunnelManager {
    pub fn new(config: TunnelConfig) -> Self {
        Self {
            local_interface: None,
            config,
        }
    }
    
    /// Initialize local WireGuard interface
    pub async fn init(&mut self) -> Result<()> {
        tracing::info!("Initializing WireGuard tunnel on port {}", self.config.listen_port);
        // TODO: Platform-specific WireGuard setup (Windows/Linux/macOS)
        Ok(())
    }
    
    /// Establish tunnel to a remote peer
    pub async fn connect(&self, peer_public_key: &str, endpoint: SocketAddr) -> Result<Tunnel> {
        tracing::info!("Connecting to peer at {} with key {}", endpoint, &peer_public_key[..8]);
        // TODO: Add peer to WireGuard, perform NAT hole punching
        Ok(Tunnel {
            peer_public_key: peer_public_key.to_string(),
            endpoint,
            status: TunnelStatus::Connecting,
        })
    }
    
    /// Close tunnel
    pub async fn disconnect(&self, tunnel: Tunnel) -> Result<()> {
        tracing::info!("Disconnecting from {}", &tunnel.peer_public_key[..8]);
        Ok(())
    }
}

/// Active P2P tunnel
#[derive(Debug)]
pub struct Tunnel {
    peer_public_key: String,
    endpoint: SocketAddr,
    status: TunnelStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunnelStatus {
    Connecting,
    Handshaking,
    Established,
    Failed,
    Closed,
}

impl Tunnel {
    pub fn status(&self) -> TunnelStatus {
        self.status
    }
    
    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tunnel_config() {
        let config = TunnelConfig {
            listen_port: 51820,
            private_key: "test_key".to_string(),
            public_key: "pub_key".to_string(),
            allowed_ips: vec!["10.0.0.0/24".to_string()],
        };
        assert_eq!(config.listen_port, 51820);
    }
}
