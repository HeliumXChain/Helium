use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};
use chrono::{DateTime, Utc};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

use crate::identity::IdentityManager;

const MESH_STATE_FILE: &str = "mesh_state.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub name: Option<String>,
    pub public_key: String,
    pub wireguard_endpoint: Option<String>,
    pub wireguard_pubkey: String,
    pub status: PeerStatus,
    pub last_seen: DateTime<Utc>,
    pub capabilities: Vec<Capability>,
    pub vouched_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerStatus {
    Connected,
    Disconnected,
    Connecting,
    Unknown,
}

impl std::fmt::Display for PeerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerStatus::Connected => write!(f, "connected"),
            PeerStatus::Disconnected => write!(f, "disconnected"),
            PeerStatus::Connecting => write!(f, "connecting"),
            PeerStatus::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Capability {
    Gpu { count: u32, model: String, vram_gb: u32 },
    Storage { capacity_gb: u64 },
    Cpu { cores: u32, memory_gb: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    pub code: String,
    pub url: String,
    pub expires_at: DateTime<Utc>,
    pub created_by: String,
    pub target_peer: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeshState {
    mesh_id: Option<String>,
    peers: HashMap<String, Peer>,
    invitations: HashMap<String, Invitation>,
    joined_at: Option<DateTime<Utc>>,
}

pub struct MeshManager {
    state: Arc<RwLock<MeshState>>,
    identity: IdentityManager,
    base_path: std::path::PathBuf,
}

impl MeshManager {
    pub async fn load() -> Result<Self> {
        let base_path = Self::helium_dir()?;
        let identity = IdentityManager::load().await?;

        let state_path = base_path.join(MESH_STATE_FILE);

        let state = if state_path.exists() {
            let state_json = tokio::fs::read_to_string(&state_path).await?;
            serde_json::from_str(&state_json).unwrap_or_else(|_| MeshState {
                mesh_id: None,
                peers: HashMap::new(),
                invitations: HashMap::new(),
                joined_at: None,
            })
        } else {
            MeshState {
                mesh_id: None,
                peers: HashMap::new(),
                invitations: HashMap::new(),
                joined_at: None,
            }
        };

        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            identity,
            base_path,
        })
    }

    pub async fn is_joined(&self) -> bool {
        let state = self.state.read().await;
        state.mesh_id.is_some()
    }

    pub async fn list_peers(&self) -> Result<Vec<Peer>> {
        let state = self.state.read().await;
        Ok(state.peers.values().cloned().collect())
    }

    pub async fn join_with_invitation(&self, invite_code: &str) -> Result<()> {
        info!("Processing invitation: {}", invite_code);

        // TODO: Parse invitation, verify signature, establish WireGuard tunnel
        // This is a stub for Phase 1

        // Release the write guard BEFORE save_state (RwLock is not reentrant).
        let mesh_id = format!("mesh-{}", uuid::Uuid::new_v4());
        {
            let mut state = self.state.write().await;
            state.mesh_id = Some(mesh_id.clone());
            state.joined_at = Some(Utc::now());
        }

        self.save_state().await?;

        info!("Successfully joined mesh {}", mesh_id);

        Ok(())
    }

    pub async fn create_invitation(&self, peer_name: &str) -> Result<Invitation> {
        let code = format!("HELIUM-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
        let node_id = self.identity.node_id()?;

        // Self-signed invitation (Phase 1). Verified on join in Phase 2.
        let signature = self.identity.sign(code.as_bytes())
            .map(|sig| URL_SAFE_NO_PAD.encode(sig.to_bytes()))
            .unwrap_or_default();

        let invitation = Invitation {
            code: code.clone(),
            url: format!("https://helium.sh/join#{}", code),
            expires_at: Utc::now() + chrono::Duration::hours(24),
            created_by: node_id.clone(),
            target_peer: peer_name.to_string(),
            signature,
        };

        {
            let mut state = self.state.write().await;
            state.invitations.insert(code.clone(), invitation.clone());
        }

        self.save_state().await?;

        info!("Created invitation {} for peer {}", code, peer_name);

        Ok(invitation)
    }

    pub async fn leave(&self) -> Result<()> {
        {
            let mut state = self.state.write().await;

            // Disconnect from all peers
            for (peer_id, peer) in state.peers.iter_mut() {
                debug!("Disconnecting from peer {}", peer_id);
                peer.status = PeerStatus::Disconnected;
                // TODO: Close WireGuard tunnels
            }

            state.mesh_id = None;
            state.joined_at = None;
        }

        self.save_state().await?;

        info!("Left mesh successfully");

        Ok(())
    }

    /// Add a peer (Phase 2: called by peer sync / invite accept).
    #[allow(dead_code)]
    pub async fn add_peer(&self, peer: Peer) -> Result<()> {
        info!("Adding peer {} to mesh", peer.id);
        {
            let mut state = self.state.write().await;
            state.peers.insert(peer.id.clone(), peer);
        }

        self.save_state().await?;

        Ok(())
    }

    /// Update peer status (Phase 2: called by peer health checks).
    #[allow(dead_code)]
    pub async fn update_peer_status(&self, peer_id: &str, status: PeerStatus) -> Result<()> {
        {
            let mut state = self.state.write().await;

            if let Some(peer) = state.peers.get_mut(peer_id) {
                peer.status = status;
                peer.last_seen = Utc::now();
            }
        }

        self.save_state().await?;

        Ok(())
    }

    async fn save_state(&self) -> Result<()> {
        let state_path = self.base_path.join(MESH_STATE_FILE);
        let state = self.state.read().await;

        let state_json = serde_json::to_string_pretty(&*state)
            .context("Failed to serialize mesh state")?;

        tokio::fs::write(&state_path, state_json).await
            .context("Failed to write mesh state")?;

        Ok(())
    }

    fn helium_dir() -> Result<std::path::PathBuf> {
        let home = dirs::home_dir()
            .context("Failed to determine home directory")?;
        Ok(home.join(".helium"))
    }
}

#[cfg(test)]
mod tests {
    // Filesystem-backed tests live in storage/market (TempDir); mesh logic
    // is covered live via CLI (join/invite/leave).
}
