use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod bench;
mod daemon;
mod discovery;
mod identity;
mod market;
mod mesh;
mod networking;
mod storage;
mod templates;
mod tui;
mod workload;

use daemon::HeliumDaemon;
use discovery::DhtDiscovery;
use identity::IdentityManager;
use market::Market;
use mesh::MeshManager;
use networking::{genkeypair, render_config, PeerConfig, WireGuardInterface};
use storage::StorageManager;

#[derive(Parser)]
#[command(name = "helium")]
#[command(about = "Helium Mesh - Private GPU sharing network")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Helium node
    Init {
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Mesh management commands
    Mesh {
        #[command(subcommand)]
        command: MeshCommands,
    },

    /// Start the Helium daemon
    Daemon {
        #[arg(short, long, default_value = "127.0.0.1:8787")]
        bind: String,
    },

    /// Show node status
    Status,

    /// Workload management
    Workload {
        #[command(subcommand)]
        command: WorkloadCommands,
    },

    /// Marketplace: offers and requests
    Market {
        #[command(subcommand)]
        command: MarketCommands,
    },

    /// Benchmark this node (sustained score, never the peak)
    Bench {
        /// Wall time in seconds (300 recommended for official scores)
        #[arg(short, long, default_value_t = 60)]
        seconds: u64,
        /// Machine-readable JSON output
        #[arg(long)]
        json: bool,
    },

    /// Show credits balance
    Credits {
        #[arg(short, long)]
        party: Option<String>,
    },

    /// Mint demo credits (no real money in MVP)
    Faucet {
        #[arg(short, long)]
        party: Option<String>,
        #[arg(short, long, default_value_t = 100.0)]
        amount: f64,
    },

    /// WireGuard tunnel management (needs root for up/down/peer)
    Tunnel {
        #[command(subcommand)]
        command: TunnelCommands,
    },

    /// Fullscreen terminal dashboard (node, peers, market, tunnel, host)
    Dash {
        /// Render one frame and exit (demos, screenshots, CI)
        #[arg(long)]
        once: bool,
    },

    /// Show this node's API token (share it with peers you trust)
    Token,

    /// Self-contained demo: faucet, offers, request, match, settle (in-memory, nothing saved)
    Demo,

    /// Remote disk: register and inspect shared datasets
    Storage {
        #[command(subcommand)]
        command: StorageCommands,
    },
}

#[derive(Subcommand)]
enum MeshCommands {
    /// Join a mesh via invitation
    Join {
        #[arg(short, long)]
        invite: String,
    },

    /// Show mesh topology and peers
    Status,

    /// Create invitation for a new peer
    Invite {
        #[arg(short, long)]
        peer: String,
    },

    /// Leave current mesh
    Leave,
}

#[derive(Subcommand)]
enum WorkloadCommands {
    /// Run a workload on available GPU
    Run {
        #[arg(short, long, default_value = "")]
        image: String,
        #[arg(short, long, default_value_t = 0)]
        gpu_count: u32,
        /// Named template (jupyter, train, batch-scan); explicit flags win
        #[arg(short, long, default_value = "")]
        template: String,
        /// Job name (default: job-<short id>)
        #[arg(short, long)]
        name: Option<String>,
    },

    /// List available job templates
    Templates,

    /// List running workloads
    List,

    /// Stop a workload
    Stop {
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Subcommand)]
enum MarketCommands {
    /// Publish a resource offer
    Offer {
        #[arg(short, long)]
        by: Option<String>,
        #[arg(short, long, default_value = "gpu")]
        rtype: String,
        #[arg(short, long, default_value_t = 16)]
        amount: u32,
        #[arg(short, long, default_value_t = 0.5)]
        price: f64,
        #[arg(short, long, default_value = "")]
        endpoint: String,
        #[arg(short, long, default_value = "")]
        wg_pubkey: String,
        /// Run `helium bench` inline and attach the sustained score
        #[arg(long)]
        bench: bool,
        /// Bench wall time in seconds (with --bench)
        #[arg(long, default_value_t = 60)]
        bench_seconds: u64,
    },

    /// List open offers
    List,

    /// List open requests
    Requests,

    /// Publish a resource request
    Request {
        #[arg(short, long)]
        by: Option<String>,
        #[arg(short, long, default_value = "gpu")]
        rtype: String,
        #[arg(short, long, default_value_t = 16)]
        amount: u32,
        #[arg(short, long, default_value_t = 1.0)]
        max_price: f64,
        /// -h est reserve a --help (clap) : pas de short ici.
        #[arg(long, default_value_t = 2)]
        hours: u32,
        #[arg(short, long, default_value = "")]
        wg_pubkey: String,
        #[arg(short, long, default_value = "")]
        endpoint: String,
        /// Container image wanted ("" = node default)
        #[arg(short, long, default_value = "")]
        image: String,
        /// GPUs wanted (0 = unset)
        #[arg(short, long, default_value_t = 0)]
        gpus: u32,
        /// Named template (jupyter, train, batch-scan); explicit flags win
        #[arg(short, long, default_value = "")]
        template: String,
        /// Bench floor: offers below this sustained GFLOPS never match
        #[arg(long, default_value_t = 0.0)]
        min_bench: f64,
    },

    /// Find matches for a request
    Match {
        #[arg(short, long)]
        request_id: String,
    },

    /// Accept a match (settles credits)
    Accept {
        #[arg(short, long)]
        match_id: String,
    },

    /// Withdraw an open request (no credits moved)
    Cancel {
        #[arg(short, long)]
        request_id: String,
    },
}

#[derive(Subcommand)]
enum TunnelCommands {
    /// Generate a keypair (no root needed)
    Keygen,

    /// Create (or recreate) the interface
    Up {
        #[arg(short, long, default_value = "helium-poc")]
        name: String,
        #[arg(short, long)]
        address: String,
        #[arg(short, long, default_value_t = 51820)]
        port: u16,
        #[arg(short, long)]
        private_key: Option<String>,
    },

    /// Delete the interface
    Down {
        #[arg(short, long, default_value = "helium-poc")]
        name: String,
    },

    /// Add or update a peer
    Peer {
        #[arg(short, long, default_value = "helium-poc")]
        name: String,
        #[arg(short, long)]
        pubkey: String,
        #[arg(short, long)]
        endpoint: Option<String>,
        #[arg(short, long, default_value = "10.0.0.0/24")]
        allowed_ips: String,
        #[arg(short, long)]
        keepalive: Option<u16>,
    },

    /// Remove a peer
    RemovePeer {
        #[arg(short, long, default_value = "helium-poc")]
        name: String,
        #[arg(short, long)]
        pubkey: String,
    },

    /// Show interface status
    Status {
        #[arg(short, long, default_value = "helium-poc")]
        name: String,
    },

    /// Print saved creation config (for backup or Windows import)
    Config {
        #[arg(short, long, default_value = "helium-poc")]
        name: String,
    },
}

#[derive(Subcommand)]
enum StorageCommands {
    /// Register a file as a chunked dataset (256MB chunks, SHA-256)
    Register {
        #[arg(short, long)]
        path: String,
        #[arg(short, long)]
        name: Option<String>,
    },

    /// List registered datasets
    List,

    /// Show dataset detail (chunks and hashes)
    Info {
        #[arg(short, long)]
        id: String,
    },

    /// Serve datasets over HTTP (for peers to fetch)
    Serve {
        #[arg(short, long, default_value_t = 8788)]
        port: u16,
    },

    /// Fetch a dataset from a peer (verified chunk by chunk)
    Fetch {
        #[arg(short, long)]
        dataset: String,
        #[arg(short, long)]
        from: String,
        #[arg(short, long, default_value = ".")]
        dir: String,
        #[arg(short, long, default_value = "")]
        token: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging (both crate names: bin `helium`, lib `helium_mesh`).
    // Logs go to stderr so stdout stays machine-readable (token, lists).
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            if cli.verbose { "helium=debug,helium_mesh=debug" } else { "helium=info,helium_mesh=info" }
        )
        .init();

    info!("Helium Mesh v{} starting...", env!("CARGO_PKG_VERSION"));

    match cli.command {
        Commands::Init { name } => {
            info!("Initializing Helium node...");
            let identity = IdentityManager::initialize(name).await?;
            println!("Node initialized successfully!");
            println!("Public Key: {}", identity.public_key_base58()?);
            println!("Config stored in: {}", identity.config_path().display());
        }

        Commands::Mesh { command } => match command {
            MeshCommands::Join { invite } => {
                info!("Joining mesh with invitation...");
                let mesh = MeshManager::load().await?;
                mesh.join_with_invitation(&invite).await?;
                println!("Successfully joined mesh!");
            }

            MeshCommands::Status => {
                let mesh = MeshManager::load().await?;
                let peers = mesh.list_peers().await?;
                println!("Mesh Status:");
                println!("  Connected peers: {}", peers.len());
                for peer in peers {
                    println!("  - {} ({})", peer.id, peer.status);
                }
            }

            MeshCommands::Invite { peer } => {
                let mesh = MeshManager::load().await?;
                let invitation = mesh.create_invitation(&peer).await?;
                println!("Invitation created: {}", invitation.code);
                println!("Share this link: {}", invitation.url);
            }

            MeshCommands::Leave => {
                let mesh = MeshManager::load().await?;
                mesh.leave().await?;
                println!("Left mesh successfully");
            }
        },

        Commands::Bench { seconds, json } => {
            let report = bench::run_bench(seconds);
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("sustained: {:.1} GFLOPS (peak {:.1}, derate {:.0}%, {}s)",
                    report.gflops_sustained, report.gflops_peak, report.derate_pct, report.seconds);
                println!("memory: {:.1} GB/s | RAM: {:.1}/{:.1} GB free | class: {}",
                    report.mem_bw_gbs, report.ram_avail_gb, report.ram_total_gb, report.suggested_class);
            }
        }

        Commands::Daemon { bind } => {
            info!("Starting Helium daemon on {}...", bind);
            let daemon = HeliumDaemon::new(bind).await?;
            daemon.run().await?;
        }

        Commands::Status => {
            let identity = IdentityManager::load().await?;
            let mesh = MeshManager::load().await?;

            println!("Node Status:");
            println!("  ID: {}", identity.node_id()?);
            println!("  Mesh: {}", if mesh.is_joined().await { "Joined" } else { "Not joined" });
            if let Ok(peers) = mesh.list_peers().await {
                println!("  Peers: {}", peers.len());
            }
            let discovered = DhtDiscovery::known_peers();
            println!("  Discovered peers (mDNS): {}", discovered.len());
            for p in discovered {
                println!("  - {} via {} (seen {})", p.peer_id, p.addresses.join(","), p.last_seen);
            }
            let kad = DhtDiscovery::kad_peers();
            println!("  DHT peers (Kademlia): {}", kad.len());
        }

        Commands::Workload { command } => match command {
            WorkloadCommands::Run { image, gpu_count, template, name } => {
                let (image, gpu_count) = templates::apply(&template, &image, gpu_count)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                let name = name.unwrap_or_else(|| format!("job-{}", &uuid::Uuid::new_v4().to_string()[..8]));
                let w = workload::WorkloadManager::new().await?
                    .create_workload(name, image, gpu_count, "", "").await?;
                println!("Workload created: {} ({}, {} gpu) [{}]", w.id, w.name, w.gpu_count, w.status);
            }
            WorkloadCommands::Templates => {
                for t in templates::list() {
                    println!("{}: {} [image={} gpus={}]", t.name, t.description, t.docker_image, t.gpus);
                }
            }
            WorkloadCommands::List => {
                let rows = workload::WorkloadManager::new().await?.list_workloads().await;
                if rows.is_empty() {
                    println!("No workloads");
                }
                for w in rows {
                    println!("{}: {} [{}] image={} gpus={}", w.id, w.name, w.status, w.docker_image, w.gpu_count);
                }
            }
            WorkloadCommands::Stop { id } => {
                workload::WorkloadManager::new().await?.stop_workload(&id).await?;
                println!("Stopped workload {id}");
            }
        },

        Commands::Market { command } => {
            let market = Market::open()?;
            match command {
                MarketCommands::Offer { by, rtype, amount, price, endpoint, wg_pubkey, bench, bench_seconds } => {
                    let by = match by {
                        Some(p) => p,
                        None => IdentityManager::load().await?.node_id()?,
                    };
                    let bench_gflops = if bench {
                        let report = bench::run_bench(bench_seconds);
                        println!("bench: sustained {:.1} GFLOPS (peak {:.1}, derate {:.0}%, {}s)",
                            report.gflops_sustained, report.gflops_peak, report.derate_pct, report.seconds);
                        report.gflops_sustained
                    } else {
                        0.0
                    };
                    let id = market.add_offer(&market::NewOffer {
                        provider: by.clone(),
                        rtype: rtype.clone(),
                        amount,
                        price,
                        endpoint,
                        wg_pubkey,
                        bench_gflops,
                    })?;
                    println!("Offer created: {id} ({rtype} {amount} @ {price}/h by {by})");
                }
                MarketCommands::List => {
                    let offers = market.list_offers()?;
                    if offers.is_empty() {
                        println!("No open offers");
                    }
                for o in offers {
                    println!("{}: {} {} @ {:.2}/h by {} bench={:.1} [{}]", o.id, o.rtype, o.amount, o.price_per_hour, o.provider, o.bench_gflops, o.status);
                }
                }
                MarketCommands::Requests => {
                    let requests = market.list_requests()?;
                    if requests.is_empty() {
                        println!("No open requests");
                    }
                    for r in requests {
                        println!("{}: {} {} max {:.2}/h x {}h by {} [{}]", r.id, r.rtype, r.amount, r.max_price, r.hours, r.requester, r.status);
                    }
                }
                MarketCommands::Request { by, rtype, amount, max_price, hours, wg_pubkey, endpoint, image, gpus, template, min_bench } => {
                    let by = match by {
                        Some(p) => p,
                        None => IdentityManager::load().await?.node_id()?,
                    };
                    if !(min_bench >= 0.0) || min_bench > 1e6 {
                        anyhow::bail!("--min-bench must be between 0 and 1000000");
                    }
                    let (image, gpus) = templates::apply(&template, &image, gpus)
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    let id = market.add_request(&market::NewRequest {
                        requester: by.clone(),
                        rtype: rtype.clone(),
                        amount,
                        max_price,
                        hours,
                        wg_pubkey,
                        endpoint,
                        image,
                        gpus,
                        min_bench,
                    })?;
                    println!("Request created: {id} ({rtype} {amount}, max {max_price}/h x {hours}h by {by})");
                }
                MarketCommands::Match { request_id } => {
                    let matches = market.find_matches(&request_id)?;
                    if matches.is_empty() {
                        println!("No matches for {request_id}");
                    }
                    for m in matches {
                        println!("{}: req {} <- offer {} score {:.2} est_cost {:.2} [{}]", m.id, m.request_id, m.offer_id, m.score, m.est_cost, m.status);
                    }
                }
                MarketCommands::Accept { match_id } => {
                    let me = IdentityManager::load().await.ok()
                        .and_then(|idm| idm.node_id().ok()).unwrap_or_default();
                    let report = crate::daemon::settle_match(&match_id, &me).await?;
                    println!("Settled: {} -> {} : {:.2} credits (workload {})",
                        report.from, report.to, report.cost, report.workload_id);
                }
                MarketCommands::Cancel { request_id } => {
                    let status = market.cancel_request(&request_id)?;
                    println!("Request {request_id}: {status}");
                }
            }
        }

        Commands::Credits { party } => {
            let market = Market::open()?;
            let party = match party {
                Some(p) => p,
                None => IdentityManager::load().await?.node_id()?,
            };
            println!("Balance {party}: {:.2} credits", market.balance(&party)?);
        }

        Commands::Faucet { party, amount } => {
            let market = Market::open()?;
            let party = match party {
                Some(p) => p,
                None => IdentityManager::load().await?.node_id()?,
            };
            let bal = market.faucet(&party, amount)?;
            println!("Minted {amount:.2} demo credits for {party} (balance: {bal:.2})");
        }

        Commands::Tunnel { command } => match command {
            TunnelCommands::Keygen => {
                let (privkey, pubkey) = genkeypair()?;
                println!("PrivateKey: {privkey}");
                println!("PublicKey: {pubkey}");
            }
            TunnelCommands::Up { name, address, port, private_key } => {
                let privkey = match private_key {
                    Some(k) => k,
                    None => {
                        let (privkey, pubkey) = genkeypair()?;
                        println!("Generated PublicKey: {pubkey}");
                        privkey
                    }
                };
                let wg = WireGuardInterface::new(&name);
                wg.up(&address, port, &privkey)?;
                // Snapshot for backup / Windows import (peers added later not included).
                if let Some(home) = dirs::home_dir() {
                    let dir = home.join(".helium");
                    let _ = std::fs::create_dir_all(&dir);
                    let path = dir.join(format!("wg-{name}.conf"));
                    if std::fs::write(&path, render_config(&privkey, &address, Some(port), &[])).is_ok() {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if let Ok(meta) = std::fs::metadata(&path) {
                                let mut perms = meta.permissions();
                                perms.set_mode(0o600);
                                let _ = std::fs::set_permissions(&path, perms);
                            }
                        }
                        println!("Config snapshot: {}", path.display());
                    }
                }
                println!("Tunnel {name} up: {address} (port {port})");
            }
            TunnelCommands::Down { name } => {
                WireGuardInterface::new(&name).down()?;
                println!("Tunnel {name} down");
            }
            TunnelCommands::Peer { name, pubkey, endpoint, allowed_ips, keepalive } => {
                let wg = WireGuardInterface::new(&name);
                wg.add_peer(&PeerConfig {
                    public_key: pubkey,
                    endpoint,
                    allowed_ips: allowed_ips.split(',').map(|s| s.trim().to_string()).collect(),
                    keepalive,
                })?;
                println!("Peer added on {name}");
            }
            TunnelCommands::Status { name } => {
                println!("{}", WireGuardInterface::new(&name).show()?);
            }
            TunnelCommands::RemovePeer { name, pubkey } => {
                WireGuardInterface::new(&name).remove_peer(&pubkey)?;
                println!("Peer removed from {name}");
            }
            TunnelCommands::Config { name } => {
                let path = dirs::home_dir()
                    .map(|h| h.join(".helium").join(format!("wg-{name}.conf")))
                    .filter(|p| p.exists())
                    .ok_or_else(|| anyhow::anyhow!("no saved config for {name} (was it created by 'helium tunnel up'?)"))?;
                println!("{}", std::fs::read_to_string(path)?);
            }
        },

        Commands::Dash { once } => {
            if once {
                tui::run_dashboard_once().await?;
            } else {
                tui::run_dashboard().await?;
            }
        }

        Commands::Token => {
            println!("{}", crate::identity::api_token());
        }

        Commands::Demo => {
            let market = Market::open_memory()?;
            println!("Helium demo — full market loop in memory (nothing saved):");
            market.faucet("demo-alice", 100.0)?;
            println!("  faucet: demo-alice +100.00");
            let o1 = market.add_offer(&market::NewOffer {
                provider: "demo-bob".to_string(), rtype: "gpu".to_string(),
                amount: 64, price: 0.9, endpoint: String::new(),
                wg_pubkey: String::new(), bench_gflops: 0.0,
            })?;
            let o2 = market.add_offer(&market::NewOffer {
                provider: "demo-carol".to_string(), rtype: "gpu".to_string(),
                amount: 16, price: 0.2, endpoint: String::new(),
                wg_pubkey: String::new(), bench_gflops: 0.0,
            })?;
            println!("  offer: {o1} (gpu 64 @ 0.90) + {o2} (gpu 16 @ 0.20)");
            let r = market.add_request(&market::NewRequest {
                requester: "demo-alice".to_string(),
                rtype: "gpu".to_string(),
                amount: 16,
                max_price: 1.0,
                hours: 1,
                wg_pubkey: String::new(),
                endpoint: String::new(),
                image: String::new(),
                gpus: 0,
                min_bench: 0.0,
            })?;
            println!("  request: {r} (gpu 16, max 1.00/h)");
            let matches = market.find_matches(&r)?;
            for m in &matches {
                println!("  match: {} score {:.2} cost {:.2}", m.id, m.score, m.est_cost);
            }
            let (from, to, cost) = market.accept(&matches[0].id)?;
            println!("  settled: {from} -> {to} : {cost:.2} credits");
            println!(
                "  balances: alice {:.2}, bob {:.2}, carol {:.2}",
                market.balance("demo-alice")?,
                market.balance("demo-bob")?,
                market.balance("demo-carol")?
            );
            println!("Done. Try it for real: helium faucet && helium market offer ...");
        }

        Commands::Storage { command } => {
            let store = StorageManager::new().await?;
            match command {
                StorageCommands::Register { path, name } => {
                    let path = std::path::PathBuf::from(&path);
                    let name = name.unwrap_or_else(|| {
                        path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "dataset".to_string())
                    });
                    let ds = store.register_file(name, &path).await?;
                    println!("Dataset registered: {} ({} bytes, {} chunks)", ds.id, ds.size_bytes, ds.chunks.len());
                }
                StorageCommands::List => {
                    let all = store.list_datasets().await;
                    if all.is_empty() {
                        println!("No datasets registered");
                    }
                    for ds in all {
                        println!("{}: {} ({} bytes, {} chunks)", ds.id, ds.name, ds.size_bytes, ds.chunks.len());
                    }
                }
                StorageCommands::Info { id } => match store.get_dataset(&id).await {
                    Some(ds) => {
                        println!("{}: {} ({} bytes)", ds.id, ds.name, ds.size_bytes);
                        for c in &ds.chunks {
                            println!("  {} {} bytes sha256:{}", c.id, c.size_bytes, c.hash);
                        }
                    }
                    None => println!("Unknown dataset: {id}"),
                },
                StorageCommands::Serve { port } => {
                    use std::sync::Arc;
                    println!("Serving datasets on 0.0.0.0:{port} (Ctrl+C to stop)");
                    Arc::new(store).serve(port).await?;
                }
                StorageCommands::Fetch { dataset, from, dir, token } => {
                    let token = if token.is_empty() {
                        std::env::var("HELIUM_API_TOKEN").unwrap_or_default()
                    } else {
                        token
                    };
                    let dest = StorageManager::fetch(
                        &from,
                        &dataset,
                        std::path::Path::new(&dir),
                        &token,
                    )
                    .await?;
                    println!("Fetched: {}", dest.display());
                }
            }
        }
    }

    Ok(())
}
