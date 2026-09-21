//! LAN discovery via mDNS (zero-config) + Kademlia DHT (WAN).
//!
//! mDNS: nodes on the same network find each other over multicast, no
//! bootstrap server. Kademlia: needs bootstrap peers
//! (`HELIUM_BOOTSTRAP=/ip4/1.2.3.4/tcp/4011/p2p/PEERID,...`), then works
//! across the internet. Fixed listen port via `HELIUM_P2P_PORT` (default:
//! random). Peer identity is stable (node Ed25519 key), persisted along
//! with discovered peers under ~/.helium/.

use anyhow::{Context, Result};
use futures::StreamExt;
use libp2p::multiaddr::Protocol;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{identity, kad, mdns, Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

const DISCOVERED_FILE: &str = "discovered.json";
const KAD_PEERS_FILE: &str = "kad_peers.json";
const PEER_ID_FILE: &str = "peer_id.json";

#[derive(NetworkBehaviour)]
struct HeliumBehaviour {
    mdns: mdns::tokio::Behaviour,
    kad: kad::Behaviour<kad::store::MemoryStore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub last_seen: String,
}

pub struct DhtDiscovery {
    node_id: String,
    local_peer_id: PeerId,
}

impl DhtDiscovery {
    /// P2P keypair derived from the node Ed25519 identity (stable across
    /// restarts). Falls back to an ephemeral keypair when no identity exists.
    async fn node_keypair() -> identity::Keypair {
        crate::identity::IdentityManager::load()
            .await
            .ok()
            .and_then(|idm| idm.signing_key().ok())
            .and_then(|sk| {
                identity::Keypair::ed25519_from_bytes(&mut sk.to_bytes().to_vec()).ok()
            })
            .unwrap_or_else(identity::Keypair::generate_ed25519)
    }

    pub async fn new(node_id: String) -> Result<Self> {
        let local_key = Self::node_keypair().await;
        let local_peer_id = PeerId::from(local_key.public());

        info!("DHT discovery initialized for node {}", node_id);
        info!("Local peer ID: {}", local_peer_id);

        Ok(Self {
            node_id,
            local_peer_id,
        })
    }

    fn discovered_path() -> Result<std::path::PathBuf> {
        let home = dirs::home_dir().context("no home directory")?;
        Ok(home.join(".helium").join(DISCOVERED_FILE))
    }

    fn load_known() -> HashMap<String, DiscoveredPeer> {
        Self::discovered_path()
            .ok()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn persist_json<T: Serialize>(name: &str, value: &T) {
        if let Some(home) = dirs::home_dir() {
            if let Ok(json) = serde_json::to_string_pretty(value) {
                let _ = std::fs::write(home.join(".helium").join(name), json);
            }
        }
    }

    fn load_json<T: serde::de::DeserializeOwned>(name: &str) -> Option<T> {
        dirs::home_dir()
            .and_then(|h| std::fs::read_to_string(h.join(".helium").join(name)).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
    }

    fn persist_known(known: &HashMap<String, DiscoveredPeer>) {
        Self::persist_json(DISCOVERED_FILE, known);
    }

    /// Split `/.../p2p/PEER` into (peer, addr-without-p2p).
    fn parse_bootstrap(addr: &Multiaddr) -> Option<(PeerId, Multiaddr)> {
        let mut base = addr.clone();
        match base.pop() {
            Some(Protocol::P2p(peer)) => Some((peer, base)),
            _ => None,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let local_key = Self::node_keypair().await;
        let local_peer_id = PeerId::from(local_key.public());
        self.local_peer_id = local_peer_id;

        Self::persist_json(
            PEER_ID_FILE,
            &serde_json::json!({ "peer_id": local_peer_id.to_string() }),
        );

        let store = kad::store::MemoryStore::new(local_peer_id);
        let behaviour = HeliumBehaviour {
            mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?,
            kad: kad::Behaviour::new(local_peer_id, store),
        };

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                Default::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )?
            .with_dns()?
            .with_behaviour(|_key| Ok(behaviour))?
            .with_swarm_config(|c| {
                c.with_idle_connection_timeout(Duration::from_secs(60))
            })
            .build();

        let port: u16 = std::env::var("HELIUM_P2P_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(0);
        swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{port}").parse()?)?;
        info!(
            "discovery live for node {} (peer {}, port {})",
            self.node_id,
            self.peer_id(),
            if port == 0 { "random".to_string() } else { port.to_string() }
        );

        // WAN bootstrap peers: /ip4/HOST/tcp/PORT/p2p/PEERID,...
        let bootstraps: Vec<String> = std::env::var("HELIUM_BOOTSTRAP")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for b in &bootstraps {
            match b.parse::<Multiaddr>().ok().and_then(|a| Self::parse_bootstrap(&a)) {
                Some((peer, addr)) => {
                    info!("bootstrap: dialling {peer} via {addr}");
                    swarm.behaviour_mut().kad.add_address(&peer, addr.clone());
                    if let Err(e) = swarm.dial(addr) {
                        warn!("bootstrap dial failed for {peer}: {e}");
                    }
                }
                None => warn!("ignoring malformed HELIUM_BOOTSTRAP entry: {b}"),
            }
        }
        if !bootstraps.is_empty() {
            if let Err(e) = swarm.behaviour_mut().kad.bootstrap() {
                warn!("kad bootstrap failed: {e}");
            }
        }

        let mut known = Self::load_known();
        let mut kad_known: HashMap<String, DiscoveredPeer> =
            Self::load_json(KAD_PEERS_FILE).unwrap_or_default();

        loop {
            match swarm.select_next_some().await {
                SwarmEvent::Behaviour(HeliumBehaviourEvent::Mdns(
                    mdns::Event::Discovered(peers),
                )) => {
                    for (peer, addr) in peers {
                        if peer == local_peer_id {
                            continue;
                        }
                        info!("discovered peer {} at {}", peer, addr);
                        known.insert(
                            peer.to_string(),
                            DiscoveredPeer {
                                peer_id: peer.to_string(),
                                addresses: vec![addr.to_string()],
                                last_seen: chrono::Utc::now().to_rfc3339(),
                            },
                        );
                        Self::persist_known(&known);
                    }
                }
                SwarmEvent::Behaviour(HeliumBehaviourEvent::Mdns(
                    mdns::Event::Expired(peers),
                )) => {
                    for (peer, _) in peers {
                        debug!("peer expired {}", peer);
                        known.remove(&peer.to_string());
                    }
                    Self::persist_known(&known);
                }
                SwarmEvent::Behaviour(HeliumBehaviourEvent::Kad(event)) => {
                    match event {
                        kad::Event::RoutingUpdated { peer, .. } => {
                            info!("kad: routing table updated via {peer}");
                            kad_known.insert(
                                peer.to_string(),
                                DiscoveredPeer {
                                    peer_id: peer.to_string(),
                                    addresses: vec![],
                                    last_seen: chrono::Utc::now().to_rfc3339(),
                                },
                            );
                            Self::persist_json(KAD_PEERS_FILE, &kad_known);
                        }
                        other => debug!("kad event: {other:?}"),
                    }
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    debug!("listening on {}", address);
                }
                _ => {}
            }
        }
    }

    pub fn peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    /// Peers discovered on the LAN (persisted across restarts).
    pub fn known_peers() -> Vec<DiscoveredPeer> {
        Self::load_known().into_values().collect()
    }

    /// Peers known to the Kademlia DHT (WAN-capable).
    pub fn kad_peers() -> Vec<DiscoveredPeer> {
        Self::load_json(KAD_PEERS_FILE)
            .map(|m: HashMap<String, DiscoveredPeer>| m.into_values().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_addr_parses() {
        let addr: Multiaddr =
            "/ip4/127.0.0.1/tcp/4011/p2p/12D3KooWPybrRYbNxuYg5ofhvrgBpRHMY2xCwgKw6LiCK5Lo9F1U"
                .parse()
                .unwrap();
        let (peer, base) = DhtDiscovery::parse_bootstrap(&addr).unwrap();
        assert_eq!(peer.to_string(), "12D3KooWPybrRYbNxuYg5ofhvrgBpRHMY2xCwgKw6LiCK5Lo9F1U");
        assert_eq!(base.to_string(), "/ip4/127.0.0.1/tcp/4011");
        assert!(DhtDiscovery::parse_bootstrap(&"/ip4/127.0.0.1/tcp/4011".parse().unwrap()).is_none());
    }
}
