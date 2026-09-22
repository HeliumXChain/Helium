use anyhow::{Context, Result};
use std::net::SocketAddr;
use tracing::{info, debug, warn, error};

use crate::identity::IdentityManager;
use crate::market::{Market, MatchRow};
use crate::mesh::MeshManager;
use crate::discovery::DhtDiscovery;

pub struct HeliumDaemon {
    bind_address: SocketAddr,
    identity: IdentityManager,
    mesh: MeshManager,
    discovery: DhtDiscovery,
}

impl HeliumDaemon {
    pub async fn new(bind: String) -> Result<Self> {
        let bind_address = bind.parse()?;
        let identity = IdentityManager::load().await?;
        let mesh = MeshManager::load().await?;
        let discovery = DhtDiscovery::new(identity.node_id()?).await?;

        Ok(Self {
            bind_address,
            identity,
            mesh,
            discovery,
        })
    }

    pub async fn run(self) -> Result<()> {
        info!("Starting Helium daemon for node {} on {}",
            self.identity.node_id().unwrap_or_else(|_| "?".to_string()),
            self.bind_address);

        // Start discovery service
        let mut discovery = self.discovery;
        tokio::spawn(async move {
            if let Err(e) = discovery.run().await {
                error!("Discovery service error: {}", e);
            }
        });

        // Start mesh maintenance
        let mesh = self.mesh;
        tokio::spawn(async move {
            Self::mesh_maintenance(mesh).await;
        });

        // Start market auto-match loop (S3 automation)
        tokio::spawn(async move {
            Self::match_loop().await;
        });

        // Start HTTP API
        let bind = self.bind_address;
        tokio::spawn(async move {
            if let Err(e) = Self::api_server(bind).await {
                error!("API server error: {}", e);
            }
        });
        info!("API listening on http://{bind}");

        // Stop on Ctrl+C / SIGTERM (normal exit flushes logs).
        info!("Helium daemon running. Press Ctrl+C to stop.");
        shutdown_signal().await;

        info!("Shutting down Helium daemon...");

        Ok(())
    }

        async fn mesh_maintenance(mesh: MeshManager) {        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

        loop {
            interval.tick().await;

            // Check peer connectivity
            if let Ok(peers) = mesh.list_peers().await {
                for peer in peers {
                    debug!("Checking peer {} (status: {})", peer.id, peer.status);
                    // TODO: Ping peers, update status
                }
            }
        }
    }

    /// HTTP API (powers the future web UI and remote monitoring).
    async fn api_server(bind: std::net::SocketAddr) -> Result<()> {
        let app = axum::Router::new()
            .route("/health", axum::routing::get(|| async { "ok" }))
            .route("/status", axum::routing::get(api_status));
        let listener = tokio::net::TcpListener::bind(bind)
            .await
            .with_context(|| format!("cannot bind API to {bind}"))?;
        axum::serve(listener, app)
            .await
            .context("API server failed")?;
        Ok(())
    }

    /// Auto-match loop: settle affordable open requests without manual CLI.
    /// Policy = the request itself (max price cap = consent). Runs every 30s.
    /// After settling, provisions the WireGuard tunnel both ways (best effort).
    async fn match_loop() {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

        loop {
            interval.tick().await;

            let market = match Market::open() {
                Ok(m) => m,
                Err(e) => {
                    debug!("auto-match: cannot open market: {e}");
                    continue;
                }
            };
            let me = crate::identity::IdentityManager::load()
                .await
                .ok()
                .and_then(|idm| idm.node_id().ok())
                .unwrap_or_default();

            let requests = match market.list_requests() {
                Ok(r) => r,
                Err(e) => {
                    debug!("auto-match: cannot list requests: {e}");
                    continue;
                }
            };
            let mut settled = 0;
            for req in requests {
                match market.find_matches(&req.id) {
                    Ok(matches) if !matches.is_empty() => {
                        let best = &matches[0];
                        match market.accept(&best.id) {
                            Ok((from, to, cost)) => {
                                info!(
                                    "auto-matched {}: {} -> {} : {:.2} credits (score {:.2})",
                                    best.id, from, to, cost, best.score
                                );
                                settled += 1;
                                Self::provision_tunnel(&market, &me, best);
                            }
                            Err(e) => debug!("auto-accept skipped for {}: {}", req.id, e),
                        }
                    }
                    Ok(_) => {}
                    Err(e) => debug!("match failed for {}: {}", req.id, e),
                }
            }
            if settled > 0 {
                info!("auto-match settled {settled} request(s)");
            }
        }
    }

    /// Configure the local WireGuard interface for an accepted match.
    /// Provider side: allocate a tunnel IP and add the borrower as peer.
    /// Borrower side: add the provider (endpoint + key from the offer).
    /// Best effort: warns and keeps going when not root or incomplete data.
    fn provision_tunnel(
        market: &Market,
        me: &str,
        m: &MatchRow,
    ) {
        use crate::networking::{PeerConfig, WireGuardInterface};

        let offer = match market.get_offer(&m.offer_id) {
            Ok(o) => o,
            Err(e) => {
                warn!("provision: {e}");
                return;
            }
        };
        let request = match market.get_request(&m.request_id) {
            Ok(r) => r,
            Err(e) => {
                warn!("provision: {e}");
                return;
            }
        };
        let wg = WireGuardInterface::new("helium-poc");
        if !wg.exists() {
            debug!("provision: no local helium-poc interface, skipping");
            return;
        }

        if offer.provider == me && !request.wg_pubkey.is_empty() {
            match market.alloc_ip(&request.wg_pubkey, &m.id) {
                Ok(ip) => {
                    let peer = PeerConfig {
                        public_key: request.wg_pubkey.clone(),
                        endpoint: if request.endpoint.is_empty() {
                            None
                        } else {
                            Some(request.endpoint.clone())
                        },
                        allowed_ips: vec![format!("{ip}/32")],
                        keepalive: None,
                    };
                    match wg.add_peer(&peer) {
                        Ok(()) => {
                            info!(
                                "tunnel ready: borrower {} -> {ip} (match {})",
                                request.requester, m.id
                            );
                            Self::maybe_boot_vm(offer.amount);
                        }
                        Err(e) => warn!("provision: cannot add borrower peer: {e}"),
                    }
                }
                Err(e) => warn!("provision: IPAM failed: {e}"),
            }
        } else if request.requester == me
            && !offer.wg_pubkey.is_empty()
            && !offer.endpoint.is_empty()
        {
            let peer = PeerConfig {
                public_key: offer.wg_pubkey.clone(),
                endpoint: Some(offer.endpoint.clone()),
                allowed_ips: vec!["10.0.0.0/24".to_string()],
                keepalive: Some(25),
            };
            match wg.add_peer(&peer) {
                Ok(()) => info!(
                    "tunnel ready: provider {} via {} (match {})",
                    offer.provider, offer.endpoint, m.id
                ),
                Err(e) => warn!("provision: cannot add provider peer: {e}"),
            }
        } else {
            debug!("provision: not a party or missing keys, skipping");
        }
    }

    /// RAM for a compute offer in MiB, capped for small providers.
    /// Offer `amount` is GiB (VRAM/RAM); floor 512MiB, cap 2048MiB.
    fn vm_mem_for_offer(amount_gb: u32) -> u64 {
        (amount_gb as u64).saturating_mul(1024).clamp(512, 2048)
    }

    /// Boot a borrower microVM on first match (background thread — slow).
    /// Uses fc-vm.sh (installed by install.sh --provider). Best effort.
    fn maybe_boot_vm(offer_amount_gb: u32) {
        use std::process::Command;

        const SCRIPT: &str = "/srv/helium-vm/fc-vm.sh";
        const SOCK: &str = "/tmp/fc-helium.socket";
        if !std::path::Path::new(SCRIPT).exists() {
            debug!("provision: no fc-vm.sh, skipping VM boot");
            return;
        }
        if std::path::Path::new(SOCK).exists() {
            debug!("provision: VM already running, skipping boot");
            return;
        }
        let mem = Self::vm_mem_for_offer(offer_amount_gb);
        std::thread::spawn(move || {
            info!("provision: booting borrower VM ({mem} MiB)…");
            let up = Command::new("sudo")
                .args(["-n", SCRIPT, "up", "--mem", &mem.to_string()])
                .output();
            match up {
                Ok(o) if o.status.success() => {
                    info!("provision: VM booted, starting demo workload");
                    let _ = Command::new("sudo")
                        .args([
                            "-n", SCRIPT, "ssh", "--",
                            "setsid nohup python3 -m http.server 8888 >/tmp/http.log 2>&1 < /dev/null & sleep 1",
                        ])
                        .output();
                    match Command::new("sudo")
                        .args(["-n", SCRIPT, "expose", "8888"])
                        .output()
                    {
                        Ok(e) if e.status.success() => {
                            info!("provision: workload reachable via tunnel :8888")
                        }
                        _ => warn!("provision: expose failed"),
                    }
                }
                _ => warn!("provision: VM boot failed (see /tmp/fc.log)"),
            }
        });
    }
}

async fn api_status(
    headers: axum::http::HeaderMap,
) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
    use crate::discovery::DhtDiscovery;
    use crate::market::Market;

    if !crate::identity::authorized(&headers, &crate::identity::api_token()) {
        return Err(axum::http::StatusCode::UNAUTHORIZED);
    }

    let node_id = IdentityManager::load()
        .await
        .ok()
        .and_then(|idm| idm.node_id().ok())
        .unwrap_or_else(|| "unknown".to_string());
    let (joined, peers) = match MeshManager::load().await {
        Ok(mesh) => (
            mesh.is_joined().await,
            mesh.list_peers().await.map(|p| p.len()).unwrap_or(0),
        ),
        Err(_) => (false, 0),
    };
    let (offers, requests) = match Market::open() {
        Ok(m) => (
            m.list_offers().map(|o| o.len()).unwrap_or(0),
            m.list_requests().map(|r| r.len()).unwrap_or(0),
        ),
        Err(_) => (0, 0),
    };
    Ok(axum::Json(serde_json::json!({
        "node": node_id,
        "mesh_joined": joined,
        "mesh_peers": peers,
        "open_offers": offers,
        "open_requests": requests,
        "discovered_lan": DhtDiscovery::known_peers().len(),
        "discovered_dht": DhtDiscovery::kad_peers().len(),
        "version": env!("CARGO_PKG_VERSION"),
    })))
}

/// Wait for Ctrl+C (all platforms) or SIGTERM (unix: systemctl stop,
/// timeout, kill). Normal exit flushes buffered logs.
#[cfg(unix)]
async fn shutdown_signal() {    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("cannot install SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = term.recv() => {},
    }
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_mem_sizing() {
        assert_eq!(HeliumDaemon::vm_mem_for_offer(0), 512); // floor
        assert_eq!(HeliumDaemon::vm_mem_for_offer(1), 1024);
        assert_eq!(HeliumDaemon::vm_mem_for_offer(2), 2048);
        assert_eq!(HeliumDaemon::vm_mem_for_offer(64), 2048); // cap
    }
}
