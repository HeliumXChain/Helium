use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, warn};

mod daemon;
mod discovery;
mod identity;
mod market;
mod mesh;
mod networking;
mod storage;
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
    Dash,

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
        #[arg(short, long)]
        image: String,
        #[arg(short, long)]
        gpu_count: u32,
    },

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
        #[arg(short, long, default_value_t = 2)]
        hours: u32,
        #[arg(short, long, default_value = "")]
        wg_pubkey: String,
        #[arg(short, long, default_value = "")]
        endpoint: String,
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
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            if cli.verbose { "helium_mesh=debug" } else { "helium_mesh=info" }
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

        Commands::Workload { .. } => {
            warn!("Workload commands not yet implemented");
            println!("Workload management coming in Phase 2");
        }

        Commands::Market { command } => {
            let market = Market::open()?;
            match command {
                MarketCommands::Offer { by, rtype, amount, price, endpoint, wg_pubkey } => {
                    let by = match by {
                        Some(p) => p,
                        None => IdentityManager::load().await?.node_id()?,
                    };
                    let id = market.add_offer(&by, &rtype, amount, price, &endpoint, &wg_pubkey)?;
                    println!("Offer created: {id} ({rtype} {amount} @ {price}/h by {by})");
                }
                MarketCommands::List => {
                    let offers = market.list_offers()?;
                    if offers.is_empty() {
                        println!("No open offers");
                    }
                    for o in offers {
                        println!("{}: {} {} @ {:.2}/h by {} [{}]", o.id, o.rtype, o.amount, o.price_per_hour, o.provider, o.status);
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
                MarketCommands::Request { by, rtype, amount, max_price, hours, wg_pubkey, endpoint } => {
                    let by = match by {
                        Some(p) => p,
                        None => IdentityManager::load().await?.node_id()?,
                    };
                    let id = market.add_request(&by, &rtype, amount, max_price, hours, &wg_pubkey, &endpoint)?;
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
                    let (from, to, cost) = market.accept(&match_id)?;
                    println!("Settled: {from} -> {to} : {cost:.2} credits");
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

        Commands::Dash => {
            tui::run_dashboard().await?;
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
                StorageCommands::Fetch { dataset, from, dir } => {
                    let dest =
                        StorageManager::fetch(&from, &dataset, std::path::Path::new(&dir)).await?;
                    println!("Fetched: {}", dest.display());
                }
            }
        }
    }

    Ok(())
}
