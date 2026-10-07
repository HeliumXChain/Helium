use anyhow::{Context, Result};
use std::net::SocketAddr;
use tracing::{info, debug, warn, error};

use crate::identity::IdentityManager;
use crate::market::{Market, MatchRow};
use crate::mesh::MeshManager;
use crate::discovery::DhtDiscovery;
use crate::workload::WorkloadManager;

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
    /// S1 (Programme F): job submission for Kuro. Everything except
    /// /health requires the node API token (`helium token` prints it).
    async fn api_server(bind: std::net::SocketAddr) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(bind)
            .await
            .with_context(|| format!("cannot bind API to {bind}"))?;
        axum::serve(listener, build_router())
            .await
            .context("API server failed")?;
        Ok(())
    }
}

/// S1 job API routes. Auth: Bearer token on everything except /health.
fn build_router() -> axum::Router {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/status", get(api_status))
        .route("/requests", post(api_create_request).get(api_list_requests))
        .route("/requests/:id", get(api_get_request))
        .route("/requests/:id/matches", get(api_request_matches))
        .route("/requests/:id/cancel", post(api_cancel_request))
        .route("/offers", get(api_list_offers))
        .route("/matches/:id/accept", post(api_accept_match))
        .route("/workloads", get(api_list_workloads))
        .route("/workloads/:id", get(api_get_workload))
        .route("/workloads/:id/logs", get(api_workload_logs))
}

type ApiOut = (axum::http::StatusCode, axum::Json<serde_json::Value>);

fn authed(headers: &axum::http::HeaderMap) -> bool {
    crate::identity::authorized(headers, &crate::identity::api_token())
}

fn unauthorized() -> ApiOut {
    (
        axum::http::StatusCode::UNAUTHORIZED,
        axum::Json(serde_json::json!({
            "error": "missing or invalid bearer token (run `helium token` on the node)"
        })),
    )
}

fn db_error(context: &str) -> ApiOut {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json(serde_json::json!({ "error": format!("{context}: cannot open local state") })),
    )
}

/// Missing row -> 404, anything else (already matched, no credits) -> 409.
fn market_status(err: &anyhow::Error) -> axum::http::StatusCode {
    let msg = err.to_string().to_lowercase();
    if msg.contains("not found") || msg.contains("no row") {
        axum::http::StatusCode::NOT_FOUND
    } else {
        axum::http::StatusCode::CONFLICT
    }
}

fn market_error(err: anyhow::Error) -> ApiOut {
    (
        market_status(&err),
        axum::Json(serde_json::json!({ "error": err.to_string() })),
    )
}

#[derive(Debug, serde::Deserialize)]
struct CreateRequestBody {
    /// Who asks (e.g. "kuro:openquant"). Empty = this node.
    #[serde(default)]
    requester: String,
    #[serde(default = "default_rtype")]
    rtype: String,
    amount: u32,
    max_price: f64,
    hours: u32,
    #[serde(default)]
    wg_pubkey: String,
    #[serde(default)]
    endpoint: String,
    /// S2: container image wanted ("" = node default, see Q5 templates).
    #[serde(default)]
    image: String,
    /// S2: GPUs wanted (0 = unset, node decides).
    #[serde(default)]
    gpus: u32,
    /// Q5: named template (jupyter, train, batch-scan); explicit fields win.
    #[serde(default)]
    template: String,
    /// Bench floor: offers below this sustained GFLOPS never match (0 = any).
    #[serde(default)]
    min_bench: f64,
}

fn default_rtype() -> String {
    "gpu".to_string()
}

/// Pure validation so it stays unit-testable without touching disk.
fn validate_request(rtype: &str, amount: u32, max_price: f64, hours: u32, image: &str, gpus: u32, template: &str, min_bench: f64) -> std::result::Result<(), String> {
    let rtype = rtype.trim();
    if rtype.is_empty() {
        return Err("rtype is required (e.g. gpu)".to_string());
    }
    if rtype.len() > 32 {
        return Err("rtype must be 32 chars or less".to_string());
    }
    if amount == 0 || amount > 1024 {
        return Err("amount must be between 1 and 1024".to_string());
    }
    if max_price.is_nan() || max_price <= 0.0 || max_price > 1000.0 {
        return Err("max_price must be greater than 0 and at most 1000".to_string());
    }
    if hours == 0 || hours > 720 {
        return Err("hours must be between 1 and 720".to_string());
    }
    if image.len() > 256 {
        return Err("image must be 256 chars or less".to_string());
    }
    if gpus > 16 {
        return Err("gpus must be 16 or less".to_string());
    }
    if image.len() > 256 {
        return Err("image must be 256 chars or less".to_string());
    }
    if template.trim().len() > 64 {
        return Err("template must be 64 chars or less".to_string());
    }
    if min_bench.is_nan() || !(0.0..=1e6).contains(&min_bench) {
        return Err("min_bench must be between 0 and 1000000".to_string());
    }
    Ok(())
}

async fn api_create_request(
    headers: axum::http::HeaderMap,
    axum::Json(body): axum::Json<CreateRequestBody>,
) -> ApiOut {
    use axum::http::StatusCode;
    if !authed(&headers) {
        return unauthorized();
    }
    if let Err(detail) = validate_request(&body.rtype, body.amount, body.max_price, body.hours, &body.image, body.gpus, &body.template, body.min_bench) {
        return (StatusCode::BAD_REQUEST, axum::Json(serde_json::json!({ "error": detail })));
    }
    let requester = if body.requester.trim().is_empty() {
        IdentityManager::load()
            .await
            .ok()
            .and_then(|idm| idm.node_id().ok())
            .unwrap_or_else(|| "api-client".to_string())
    } else {
        body.requester.trim().to_string()
    };
    let market = match Market::open() {
        Ok(m) => m,
        Err(_) => return db_error("market"),
    };
    let (image, gpus) = match crate::templates::apply(&body.template, &body.image, body.gpus) {
        Ok(v) => v,
        Err(detail) => {
            return (StatusCode::BAD_REQUEST, axum::Json(serde_json::json!({ "error": detail })))
        }
    };
    let id = match market.add_request(&crate::market::NewRequest {
        requester,
        rtype: body.rtype.trim().to_string(),
        amount: body.amount,
        max_price: body.max_price,
        hours: body.hours,
        wg_pubkey: body.wg_pubkey.clone(),
        endpoint: body.endpoint.clone(),
        image,
        gpus,
        min_bench: body.min_bench,
    }) {
        Ok(id) => id,
        Err(e) => return market_error(e),
    };
    match market.get_request(&id) {
        Ok(req) => (
            StatusCode::CREATED,
            axum::Json(serde_json::to_value(&req).unwrap_or(serde_json::Value::Null)),
        ),
        Err(e) => market_error(e),
    }
}

async fn api_list_requests(headers: axum::http::HeaderMap) -> ApiOut {
    use axum::http::StatusCode;
    if !authed(&headers) {
        return unauthorized();
    }
    let market = match Market::open() {
        Ok(m) => m,
        Err(_) => return db_error("market"),
    };
    match market.list_requests() {
        Ok(rows) => (
            StatusCode::OK,
            axum::Json(serde_json::to_value(&rows).unwrap_or(serde_json::Value::Null)),
        ),
        Err(e) => market_error(e),
    }
}

async fn api_get_request(
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
) -> ApiOut {
    use axum::http::StatusCode;
    if !authed(&headers) {
        return unauthorized();
    }
    let market = match Market::open() {
        Ok(m) => m,
        Err(_) => return db_error("market"),
    };
    match market.get_request(&id) {
        Ok(req) => (
            StatusCode::OK,
            axum::Json(serde_json::to_value(&req).unwrap_or(serde_json::Value::Null)),
        ),
        Err(e) => market_error(e),
    }
}

async fn api_list_offers(headers: axum::http::HeaderMap) -> ApiOut {
    use axum::http::StatusCode;
    if !authed(&headers) {
        return unauthorized();
    }
    let market = match Market::open() {
        Ok(m) => m,
        Err(_) => return db_error("market"),
    };
    match market.list_offers() {
        Ok(rows) => (
            StatusCode::OK,
            axum::Json(serde_json::to_value(&rows).unwrap_or(serde_json::Value::Null)),
        ),
        Err(e) => market_error(e),
    }
}

async fn api_request_matches(
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
) -> ApiOut {
    use axum::http::StatusCode;
    if !authed(&headers) {
        return unauthorized();
    }
    let market = match Market::open() {
        Ok(m) => m,
        Err(_) => return db_error("market"),
    };
    match market.find_matches(&id) {
        Ok(rows) => (
            StatusCode::OK,
            axum::Json(serde_json::to_value(&rows).unwrap_or(serde_json::Value::Null)),
        ),
        Err(e) => market_error(e),
    }
}

async fn api_accept_match(
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
) -> ApiOut {
    use axum::http::StatusCode;
    if !authed(&headers) {
        return unauthorized();
    }
    let me = IdentityManager::load()
        .await
        .ok()
        .and_then(|idm| idm.node_id().ok())
        .unwrap_or_default();
    match settle_match(&id, &me).await {
        Ok(report) => (
            StatusCode::OK,
            axum::Json(serde_json::to_value(&report).unwrap_or(serde_json::Value::Null)),
        ),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("quota exceeded") {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    axum::Json(serde_json::json!({ "error": msg })),
                )
            } else if msg.contains("recording workload")
                || msg.contains("opening market")
                || msg.contains("opening workloads")
            {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": msg })),
                )
            } else {
                market_error(e)
            }
        }
    }
}

async fn api_cancel_request(
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
) -> ApiOut {
    use axum::http::StatusCode;
    if !authed(&headers) {
        return unauthorized();
    }
    let market = match Market::open() {
        Ok(m) => m,
        Err(_) => return db_error("market"),
    };
    match market.cancel_request(&id) {
        Ok(status) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "id": id, "status": status })),
        ),
        Err(e) => market_error(e),
    }
}

async fn api_list_workloads(headers: axum::http::HeaderMap) -> ApiOut {
    use axum::http::StatusCode;
    if !authed(&headers) {
        return unauthorized();
    }
    let manager = match WorkloadManager::new().await {
        Ok(m) => m,
        Err(_) => return db_error("workloads"),
    };
    (
        StatusCode::OK,
        axum::Json(
            serde_json::to_value(&manager.list_workloads().await)
                .unwrap_or(serde_json::Value::Null),
        ),
    )
}

async fn api_get_workload(
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
) -> ApiOut {
    use axum::http::StatusCode;
    if !authed(&headers) {
        return unauthorized();
    }
    let manager = match WorkloadManager::new().await {
        Ok(m) => m,
        Err(_) => return db_error("workloads"),
    };
    match manager.get_workload(&id).await {
        Some(w) => (
            StatusCode::OK,
            axum::Json(serde_json::to_value(&w).unwrap_or(serde_json::Value::Null)),
        ),
        None => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": format!("workload {id} not found") })),
        ),
    }
}

async fn api_workload_logs(
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
) -> ApiOut {
    use axum::http::StatusCode;
    if !authed(&headers) {
        return unauthorized();
    }
    let manager = match WorkloadManager::new().await {
        Ok(m) => m,
        Err(_) => return db_error("workloads"),
    };
    match manager.get_workload(&id).await {
        Some(w) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "id": w.id,
                "status": w.status.to_string(),
                "logs": w.logs,
            })),
        ),
        None => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": format!("workload {id} not found") })),
        ),
    }
}

/// Background loops (second half of the daemon impl).
impl HeliumDaemon {
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
            // S3: evict open requests older than the TTL so the queue cannot rot.
            match market.expire_stale(request_ttl_hours()) {
                Ok(0) => {}
                Ok(n) => info!("auto-match expired {n} stale request(s)"),
                Err(e) => debug!("auto-match: cannot expire stale requests: {e}"),
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
                        match settle_match(&best.id, &me).await {
                            Ok(report) => {
                                info!(
                                    "auto-matched {}: {} -> {} : {:.2} credits (workload {})",
                                    report.match_id, report.from, report.to,
                                    report.cost, report.workload_id
                                );
                                settled += 1;
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
    /// Returns a one-line outcome for workload logs (S2).
    fn provision_tunnel(
        market: &Market,
        me: &str,
        m: &MatchRow,
    ) -> String {
        use crate::networking::{PeerConfig, WireGuardInterface};

        let offer = match market.get_offer(&m.offer_id) {
            Ok(o) => o,
            Err(e) => {
                warn!("provision: {e}");
                return format!("tunnel skipped: {e}");
            }
        };
        let request = match market.get_request(&m.request_id) {
            Ok(r) => r,
            Err(e) => {
                warn!("provision: {e}");
                return format!("tunnel skipped: {e}");
            }
        };
        let wg = WireGuardInterface::new("helium-poc");
        if !wg.exists() {
            debug!("provision: no local helium-poc interface, skipping");
            return "tunnel skipped: no local helium-poc interface".to_string();
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
                            format!("tunnel ready: borrower {} -> {ip} (match {})", request.requester, m.id)
                        }
                        Err(e) => {
                            warn!("provision: cannot add borrower peer: {e}");
                            format!("tunnel skipped: cannot add borrower peer: {e}")
                        }
                    }
                }
                Err(e) => {
                    warn!("provision: IPAM failed: {e}");
                    format!("tunnel skipped: IPAM failed: {e}")
                }
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
                Ok(()) => {
                    info!(
                        "tunnel ready: provider {} via {} (match {})",
                        offer.provider, offer.endpoint, m.id
                    );
                    format!("tunnel ready: provider {} via {} (match {})", offer.provider, offer.endpoint, m.id)
                }
                Err(e) => {
                    warn!("provision: cannot add provider peer: {e}");
                    format!("tunnel skipped: cannot add provider peer: {e}")
                }
            }
        } else {
            debug!("provision: not a party or missing keys, skipping");
            "tunnel skipped: not a party or missing keys".to_string()
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

/// Image recorded when the request names none. Real templates arrive in Q5.
const DEFAULT_JOB_IMAGE: &str = "helium/job:latest";

/// S3 scheduler knobs (env-overridable, documented in the S3 demo scripts).
const DEFAULT_QUOTA_JOBS_PER_REQUESTER: u32 = 2;
const DEFAULT_REQUEST_TTL_HOURS: u32 = 24;

fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn quota_jobs_per_requester() -> u32 {
    env_u32("HELIUM_MAX_ACTIVE_JOBS_PER_REQUESTER", DEFAULT_QUOTA_JOBS_PER_REQUESTER)
}

fn request_ttl_hours() -> u32 {
    env_u32("HELIUM_REQUEST_TTL_HOURS", DEFAULT_REQUEST_TTL_HOURS)
}

/// S2: outcome of settling one match, shared by the CLI, the API and the loop.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SettleReport {
    pub match_id: String,
    pub from: String,
    pub to: String,
    pub cost: f64,
    pub workload_id: String,
    pub workload_status: String,
    pub tunnel: String,
}

/// Accept a proposed match, provision the tunnel and record the workload.
/// Single entry point so CLI, API and auto-match behave the same (S2).
/// Opens its own market connection: Market is Send but not Sync, so no
/// shared reference may cross an await (tokio::spawn + axum need Send).
/// Workload-stage failures carry a "recording workload" context so the API
/// can answer 500 instead of a misleading 409.
pub async fn settle_match(match_id: &str, me: &str) -> Result<SettleReport> {
    let market = Market::open().context("opening market")?;
    let m = market.get_match(match_id)?;
    let request = market.get_request(&m.request_id)?;
    // S3 quota first: reject before any credit moves, request stays open.
    let manager = WorkloadManager::new()
        .await
        .context("opening workloads")?;
    let quota = quota_jobs_per_requester();
    let active = manager.count_active_for(&request.requester).await;
    if active >= quota as usize {
        anyhow::bail!(
            "quota exceeded for {}: {active}/{quota} active jobs (HELIUM_MAX_ACTIVE_JOBS_PER_REQUESTER)",
            request.requester
        );
    }
    let (from, to, cost) = market.accept(match_id)?;
    let tunnel = HeliumDaemon::provision_tunnel(&market, me, &m);
    let image = if request.image.trim().is_empty() {
        DEFAULT_JOB_IMAGE.to_string()
    } else {
        request.image.clone()
    };
    let workload = manager
        .create_workload(
            format!("job-{}", request.id),
            image,
            request.gpus,
            &request.id,
            &request.requester,
        )
        .await
        .context("recording workload: cannot create")?;
    let line = format!("matched {match_id} ({} -> {} : {:.2} credits); {}", from, to, cost, tunnel);
    manager
        .mark_running(&workload.id, line)
        .await
        .context("recording workload: cannot mark running")?;
    Ok(SettleReport {
        match_id: match_id.to_string(),
        from,
        to,
        cost,
        workload_id: workload.id,
        workload_status: "running".to_string(),
        tunnel,
    })
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
    let (workloads_pending, workloads_running) = match WorkloadManager::new().await {
        Ok(mgr) => {
            let mut pending = 0;
            let mut running = 0;
            for w in mgr.list_workloads().await {
                match w.status {
                    crate::workload::WorkloadStatus::Pending => pending += 1,
                    crate::workload::WorkloadStatus::Running => running += 1,
                    _ => {}
                }
            }
            (pending, running)
        }
        Err(_) => (0, 0),
    };
    Ok(axum::Json(serde_json::json!({
        "node": node_id,
        "mesh_joined": joined,
        "mesh_peers": peers,
        "open_offers": offers,
        "open_requests": requests,
        "workloads_pending": workloads_pending,
        "workloads_running": workloads_running,
        "quota_jobs_per_requester": quota_jobs_per_requester(),
        "request_ttl_hours": request_ttl_hours(),
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

    /// Process-global serial lock for env-mutating HTTP tests: HELIUM_HOME
    /// (like HOME) is shared by all test threads in the process.
    static ISOLATED_HOME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Redirect node state to a temp dir (dirs ignores process env on
    /// Windows, so HELIUM_HOME exists for exactly this). The lock guard MUST
    /// stay alive until after restore (declare it first so it drops last).
    type HomeGuard = (
        tempfile::TempDir,
        std::sync::MutexGuard<'static, ()>,
        Vec<(String, Option<std::ffi::OsString>)>,
    );

    fn lock_isolated_home() -> HomeGuard {
        let guard = ISOLATED_HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().expect("tempdir for isolated HOME");
        let keys = ["HELIUM_HOME", "HOME", "USERPROFILE"];
        let old: Vec<(String, Option<std::ffi::OsString>)> = keys
            .iter()
            .map(|k| (k.to_string(), std::env::var_os(k)))
            .collect();
        std::env::set_var("HELIUM_HOME", tmp.path());
        std::env::set_var("HOME", tmp.path());
        std::env::set_var("USERPROFILE", tmp.path());
        (tmp, guard, old)
    }

    fn restore_home(old: Vec<(String, Option<std::ffi::OsString>)>) {
        for (k, v) in old {
            match v {
                Some(x) => std::env::set_var(&k, x),
                None => std::env::remove_var(&k),
            }
        }
    }

    fn new_offer(provider: &str) -> crate::market::NewOffer {
        crate::market::NewOffer {
            provider: provider.to_string(),
            rtype: "gpu".to_string(),
            amount: 64,
            price: 0.5,
            endpoint: String::new(),
            wg_pubkey: String::new(),
            bench_gflops: 0.0,
        }
    }

    #[test]
    fn vm_mem_sizing() {
        assert_eq!(HeliumDaemon::vm_mem_for_offer(0), 512); // floor
        assert_eq!(HeliumDaemon::vm_mem_for_offer(1), 1024);
        assert_eq!(HeliumDaemon::vm_mem_for_offer(2), 2048);
        assert_eq!(HeliumDaemon::vm_mem_for_offer(64), 2048); // cap
    }

    #[test]
    fn s1_request_validation() {
        assert!(validate_request("gpu", 16, 1.0, 2, "", 0, "", 0.0).is_ok());
        assert!(validate_request("ram", 1, 0.01, 1, "ubuntu:22.04", 1, "", 0.0).is_ok());
        assert!(validate_request("gpu", 16, 1.0, 2, "", 0, "jupyter", 0.0).is_ok());
        assert!(validate_request("", 16, 1.0, 2, "", 0, "", 0.0).is_err()); // missing rtype
        assert!(validate_request("   ", 16, 1.0, 2, "", 0, "", 0.0).is_err());
        assert!(validate_request("gpu", 0, 1.0, 2, "", 0, "", 0.0).is_err()); // amount 0
        assert!(validate_request("gpu", 1025, 1.0, 2, "", 0, "", 0.0).is_err());
        assert!(validate_request("gpu", 16, 0.0, 2, "", 0, "", 0.0).is_err()); // free
        assert!(validate_request("gpu", 16, -1.0, 2, "", 0, "", 0.0).is_err());
        assert!(validate_request("gpu", 16, f64::NAN, 2, "", 0, "", 0.0).is_err());
        assert!(validate_request("gpu", 16, 1000.01, 2, "", 0, "", 0.0).is_err());
        assert!(validate_request("gpu", 16, 1.0, 0, "", 0, "", 0.0).is_err()); // hours 0
        assert!(validate_request("gpu", 16, 1.0, 721, "", 0, "", 0.0).is_err());
        assert!(validate_request("gpu", 16, 1.0, 2, &"x".repeat(257), 0, "", 0.0).is_err()); // image long
        assert!(validate_request("gpu", 16, 1.0, 2, "", 17, "", 0.0).is_err()); // too many gpus
        assert!(validate_request("gpu", 16, 1.0, 2, "", 0, &"t".repeat(65), 0.0).is_err()); // template long
        assert!(validate_request("gpu", 16, 1.0, 2, "", 0, "", -1.0).is_err()); // min_bench negative
        assert!(validate_request("gpu", 16, 1.0, 2, "", 0, "", f64::NAN).is_err());
    }

    #[test]
    fn s1_market_error_mapping() {
        use axum::http::StatusCode;
        let not_found = anyhow::anyhow!("request req-abcdef12 not found");
        assert_eq!(market_status(&not_found), StatusCode::NOT_FOUND);
        let no_rows = anyhow::anyhow!("Query returned no rows");
        assert_eq!(market_status(&no_rows), StatusCode::NOT_FOUND);
        let taken = anyhow::anyhow!("match match-abc is not proposed (status: accepted)");
        assert_eq!(market_status(&taken), StatusCode::CONFLICT);
        let broke = anyhow::anyhow!("insufficient credits for alice: 0.00 < 1.00");
        assert_eq!(market_status(&broke), StatusCode::CONFLICT);
    }

    /// S1 proof over real HTTP: 401 without token, 201 on valid request,
    /// 400 on invalid values, 404 on unknown ids, full match/accept loop.
    /// Runs against an isolated HOME so no real node state is touched.
    /// Test-only serial lock: no deadlock (holders never wait on each other
    /// while holding), but it parks a worker thread, hence the allow.
    #[allow(clippy::await_holding_lock)]
    #[tokio::test]
    async fn s1_job_api_http() {
        let (_tmp, _home_guard, old_env) = lock_isolated_home();

        let token = crate::identity::api_token();
        assert!(!token.is_empty());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, build_router()).await
        });
        let base = format!("http://{addr}");
        let client = reqwest::Client::new();

        // No token -> 401, token -> 200 (health stays open by design).
        let r_health = client.get(format!("{base}/health")).send().await.expect("health");
        let r_status_anon = client.get(format!("{base}/status")).send().await.expect("status anon");
        let r_status_auth = client
            .get(format!("{base}/status"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("status auth");
        let r_create_anon = client
            .post(format!("{base}/requests"))
            .json(&serde_json::json!({"rtype": "gpu", "amount": 16, "max_price": 1.0, "hours": 2}))
            .send()
            .await
            .expect("create anon");
        // Invalid values with token -> 400, never 201.
        let r_create_bad = client
            .post(format!("{base}/requests"))
            .bearer_auth(&token)
            .json(&serde_json::json!({"rtype": "gpu", "amount": 0, "max_price": 1.0, "hours": 2}))
            .send()
            .await
            .expect("create bad");
        // Valid request -> 201 with an id.
        let r_create = client
            .post(format!("{base}/requests"))
            .bearer_auth(&token)
            .json(&serde_json::json!({
                "requester": "kuro:test", "rtype": "gpu",
                "amount": 16, "max_price": 1.0, "hours": 2,
            }))
            .send()
            .await
            .expect("create");
        let created: serde_json::Value = r_create.json().await.expect("create body");
        let req_id = created.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let r_get = client
            .get(format!("{base}/requests/{req_id}"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("get request");
        let r_get_missing = client
            .get(format!("{base}/requests/req-nope1234"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("get missing");
        let r_offers = client
            .get(format!("{base}/offers"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("offers");
        let r_workloads = client
            .get(format!("{base}/workloads"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("workloads");
        let r_workload_missing = client
            .get(format!("{base}/workloads/wl-nope1234"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("workload missing");
        let r_accept_missing = client
            .post(format!("{base}/matches/match-nope12/accept"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("accept missing");

        // Full loop: provider offer + credits, then match + accept over HTTP.
        let market = Market::open().expect("open market in isolated HOME");
        market.faucet("kuro:test", 100.0).expect("faucet");
        market.add_offer(&new_offer("provider:test")).expect("offer");
        let r_matches = client
            .get(format!("{base}/requests/{req_id}/matches"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("matches");
        let matches: serde_json::Value = r_matches.json().await.expect("matches body");
        let match_id = matches.get(0).and_then(|m| m.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let r_accept = client
            .post(format!("{base}/matches/{match_id}/accept"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("accept");

        server.abort();
        restore_home(old_env);

        assert_eq!(r_health.status().as_u16(), 200);
        assert_eq!(r_status_anon.status().as_u16(), 401);
        assert_eq!(r_status_auth.status().as_u16(), 200);
        assert_eq!(r_create_anon.status().as_u16(), 401);
        assert_eq!(r_create_bad.status().as_u16(), 400);
        assert!(!req_id.is_empty(), "201 body must carry the request id: {created}");
        assert_eq!(r_get.status().as_u16(), 200);
        assert_eq!(r_get_missing.status().as_u16(), 404);
        assert_eq!(r_offers.status().as_u16(), 200);
        assert_eq!(r_workloads.status().as_u16(), 200);
        assert_eq!(r_workload_missing.status().as_u16(), 404);
        assert_eq!(r_accept_missing.status().as_u16(), 404);
        assert!(!match_id.is_empty(), "matches must list the test offer: {matches}");
        assert_eq!(r_accept.status().as_u16(), 200);
    }

    /// S2 proof: accepting a match records a Running workload carrying the
    /// requested image/gpus, with the tunnel outcome in its first log line
    /// (ready, or skipped with the reason — no silent skip).
    /// Test-only serial lock: no deadlock (holders never wait on each other
    /// while holding), but it parks a worker thread, hence the allow.
    #[allow(clippy::await_holding_lock)]
    #[tokio::test]
    async fn s2_accept_creates_workload() {
        let (_tmp, _home_guard, old_env) = lock_isolated_home();

        let token = crate::identity::api_token();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, build_router()).await
        });
        let base = format!("http://{addr}");
        let client = reqwest::Client::new();

        let r_create = client
            .post(format!("{base}/requests"))
            .bearer_auth(&token)
            .json(&serde_json::json!({
                "requester": "kuro:s2", "rtype": "gpu",
                "amount": 16, "max_price": 1.0, "hours": 2,
                "image": "ubuntu:22.04", "gpus": 1,
            }))
            .send()
            .await
            .expect("create");
        let created: serde_json::Value = r_create.json().await.expect("create body");
        let req_id = created.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let market = Market::open().expect("open market in isolated HOME");
        market.faucet("kuro:s2", 100.0).expect("faucet");
        market.add_offer(&new_offer("provider:s2")).expect("offer");

        let r_matches = client
            .get(format!("{base}/requests/{req_id}/matches"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("matches");
        let matches: serde_json::Value = r_matches.json().await.expect("matches body");
        let match_id = matches.get(0).and_then(|m| m.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();

        let r_accept = client
            .post(format!("{base}/matches/{match_id}/accept"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("accept");
        let settled: serde_json::Value = r_accept.json().await.expect("accept body");
        let workload_id = settled.get("workload_id").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let r_workloads = client
            .get(format!("{base}/workloads"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("workloads");
        let workloads: serde_json::Value = r_workloads.json().await.expect("workloads body");
        let r_logs = client
            .get(format!("{base}/workloads/{workload_id}/logs"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("logs");
        let logs: serde_json::Value = r_logs.json().await.expect("logs body");

        server.abort();
        restore_home(old_env);

        assert!(!req_id.is_empty(), "201 must carry the request id");
        assert!(!match_id.is_empty(), "matches must list the test offer");
        assert!(!workload_id.is_empty(), "accept must carry the workload id: {settled}");
        assert_eq!(settled.get("workload_status").and_then(|v| v.as_str()), Some("running"));
        let mine = workloads.as_array().expect("workloads array").iter()
            .find(|w| w.get("id").and_then(|v| v.as_str()) == Some(workload_id.as_str()))
            .expect("workload listed");
        assert_eq!(mine.get("status").and_then(|v| v.as_str()), Some("Running"));
        assert_eq!(mine.get("docker_image").and_then(|v| v.as_str()), Some("ubuntu:22.04"));
        assert_eq!(mine.get("gpu_count").and_then(|v| v.as_u64()), Some(1));
        assert!(mine.get("name").and_then(|v| v.as_str()).unwrap_or("").contains(&req_id));
        let lines = logs.get("logs").and_then(|v| v.as_array()).expect("logs array");
        assert!(!lines.is_empty(), "workload must log the tunnel outcome");
        assert!(lines[0].as_str().unwrap_or("").contains("tunnel"), "first log explains the tunnel: {lines:?}");
    }

    /// S3 proof over real HTTP with the default quota (2 active jobs):
    /// two accepts run, the third is rejected 429 with a clear message,
    /// cancel withdraws the request, and /status reports the scheduler.
    /// Test-only serial lock: no deadlock (holders never wait on each other
    /// while holding), but it parks a worker thread, hence the allow.
    #[allow(clippy::await_holding_lock)]
    #[tokio::test]
    async fn s3_quota_cancel_and_status() {
        let (_tmp, _home_guard, old_env) = lock_isolated_home();

        let token = crate::identity::api_token();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, build_router()).await
        });
        let base = format!("http://{addr}");
        let client = reqwest::Client::new();
        let authed = |b: reqwest::RequestBuilder| b.bearer_auth(&token);

        let mut req_ids = Vec::new();
        for _ in 0..3 {
            let created: serde_json::Value = authed(client
                .post(format!("{base}/requests"))
                .json(&serde_json::json!({
                    "requester": "kuro:s3", "rtype": "gpu",
                    "amount": 8, "max_price": 2.0, "hours": 1,
                })))
                .send().await.expect("create").json().await.expect("create body");
            req_ids.push(created.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string());
        }

        let market = Market::open().expect("open market in isolated HOME");
        market.faucet("kuro:s3", 1000.0).expect("faucet");
        market.add_offer(&new_offer("provider:s3")).expect("offer");
        market.add_offer(&new_offer("provider:s3b")).expect("offer");
        market.add_offer(&new_offer("provider:s3c")).expect("offer");

        let mut statuses = Vec::new();
        for rid in &req_ids {
            let matches: serde_json::Value = authed(client
                .get(format!("{base}/requests/{rid}/matches")))
                .send().await.expect("matches").json().await.expect("matches body");
            let mid = matches.get(0).and_then(|m| m.get("id")).and_then(|v| v.as_str()).unwrap_or("");
            let resp = authed(client
                .post(format!("{base}/matches/{mid}/accept")))
                .send().await.expect("accept");
            statuses.push(resp.status().as_u16());
        }

        // Third request: cancel it, then every settle path must refuse.
        let r_cancel = authed(client
            .post(format!("{base}/requests/{}/cancel", req_ids[2])))
            .send().await.expect("cancel");
        let cancelled: serde_json::Value = r_cancel.json().await.expect("cancel body");
        let r_get = authed(client
            .get(format!("{base}/requests/{}", req_ids[2])))
            .send().await.expect("get cancelled");
        let got: serde_json::Value = r_get.json().await.expect("get body");
        let r_matches_after = authed(client
            .get(format!("{base}/requests/{}/matches", req_ids[2])))
            .send().await.expect("matches after cancel");
        let r_status = authed(client
            .get(format!("{base}/status")))
            .send().await.expect("status");
        let status: serde_json::Value = r_status.json().await.expect("status body");

        server.abort();
        restore_home(old_env);

        assert!(req_ids.iter().all(|id| !id.is_empty()), "201 must carry ids");
        assert_eq!(statuses, vec![200, 200, 429], "third concurrent job hits the quota: {statuses:?}");
        assert_eq!(cancelled.get("status").and_then(|v| v.as_str()), Some("cancelled"));
        assert_eq!(got.get("status").and_then(|v| v.as_str()), Some("cancelled"));
        assert_eq!(r_matches_after.status().as_u16(), 409, "cancelled requests match nothing");
        assert_eq!(status.get("workloads_running").and_then(|v| v.as_u64()), Some(2));
        assert_eq!(status.get("quota_jobs_per_requester").and_then(|v| v.as_u64()), Some(2));
    }

    /// Q5 proof: template=jupyter resolves server-side (image+gpus recorded),
    /// explicit fields win, unknown names fail 400 with the known list.
    #[allow(clippy::await_holding_lock)]
    #[tokio::test]
    async fn q5_template_resolves_over_http() {
        let (_tmp, _home_guard, old_env) = lock_isolated_home();
        let token = crate::identity::api_token();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, build_router()).await
        });
        let base = format!("http://{addr}");
        let client = reqwest::Client::new();
        let authed = |b: reqwest::RequestBuilder| b.bearer_auth(&token);

        let r_tpl: serde_json::Value = authed(client
            .post(format!("{base}/requests"))
            .json(&serde_json::json!({
                "requester": "kuro:q5", "rtype": "gpu",
                "amount": 8, "max_price": 2.0, "hours": 1,
                "template": "jupyter",
            })))
            .send().await.expect("create template").json().await.expect("body");
        let r_over: serde_json::Value = authed(client
            .post(format!("{base}/requests"))
            .json(&serde_json::json!({
                "requester": "kuro:q5", "rtype": "gpu",
                "amount": 8, "max_price": 2.0, "hours": 1,
                "template": "jupyter", "image": "custom:9", "gpus": 4,
            })))
            .send().await.expect("create override").json().await.expect("body");
        let r_bad = authed(client
            .post(format!("{base}/requests"))
            .json(&serde_json::json!({
                "requester": "kuro:q5", "rtype": "gpu",
                "amount": 8, "max_price": 2.0, "hours": 1,
                "template": "nope",
            })))
            .send().await.expect("create bad");

        server.abort();
        restore_home(old_env);

        assert_eq!(r_tpl.get("image").and_then(|v| v.as_str()), Some("jupyter/pytorch-notebook:latest"));
        assert_eq!(r_tpl.get("gpus").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(r_over.get("image").and_then(|v| v.as_str()), Some("custom:9"));
        assert_eq!(r_over.get("gpus").and_then(|v| v.as_u64()), Some(4));
        assert_eq!(r_bad.status().as_u16(), 400);
    }
}
