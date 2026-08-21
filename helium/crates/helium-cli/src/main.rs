//! Helium CLI - Command Line Interface
//!
//! Main entry point for the Helium P2P compute marketplace.
//! Output style: no emojis (R9), colored prefixes, clean tables.

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;
use helium_core::{HeliumNode};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tracing::info;

// ---- Output helpers (R9: no emojis) ----

fn ok(msg: &str) {
    println!("{} {}", style("[OK]").green().bold(), msg);
}

fn err(msg: &str) {
    eprintln!("{} {}", style("[ERR]").red().bold(), msg);
}

fn info_line(msg: &str) {
    println!("{} {}", style("[..]").cyan().bold(), msg);
}

fn warn_line(msg: &str) {
    println!("{} {}", style("[WARN]").yellow().bold(), msg);
}

fn header(msg: &str) {
    println!("{}", style(msg).bold().underlined());
}

#[derive(Parser)]
#[command(name = "helium")]
#[command(about = "Helium - P2P compute sharing between trusted peers")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration directory
    #[arg(short, long, default_value = "~/.helium")]
    config_dir: PathBuf,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Helium configuration
    Init {
        /// Node type: provider or borrower
        #[arg(short, long)]
        node_type: String,
    },

    /// Start the Helium node
    Start {
        /// Run in background
        #[arg(short, long)]
        daemon: bool,
    },

    /// Stop the Helium node
    Stop,

    /// Advertise resources (provider only)
    Advertise {
        /// Resource type: cpu, ram, gpu
        #[arg(short, long)]
        resource_type: String,

        /// Resource capacity
        #[arg(short, long)]
        capacity: u32,

        /// Price per hour
        #[arg(short, long)]
        price: f64,
    },

    /// Request resources (borrower only)
    Request {
        /// Resource type: cpu, ram, gpu
        #[arg(short, long)]
        resource_type: String,

        /// Minimum capacity needed
        #[arg(short, long)]
        min_capacity: u32,

        /// Maximum price per hour
        #[arg(short, long)]
        max_price: f64,

        /// Duration needed (hours)
        #[arg(short, long)]
        duration: u32,
    },

    /// List available peers and resources
    Peers,

    /// Connect to a provider (borrower only)
    Connect {
        /// Provider peer ID
        #[arg(short, long)]
        peer_id: String,
    },

    /// VM management commands
    Vm {
        #[command(subcommand)]
        command: VmCommands,
    },

    /// Tunnel management commands
    Tunnel {
        #[command(subcommand)]
        command: TunnelCommands,
    },

    /// Check node status
    Status,

    /// Run diagnostics
    Doctor,
}

#[derive(Subcommand)]
enum VmCommands {
    /// List active VMs
    List,

    /// Create a new VM
    Create {
        #[arg(short, long)]
        vcpus: u32,
        #[arg(short, long)]
        memory: u32,
    },

    /// Start a VM
    Start { vm_id: String },

    /// Stop a VM
    Stop { vm_id: String },

    /// Kill a VM (force stop)
    Kill { vm_id: String },
}

#[derive(Subcommand)]
enum TunnelCommands {
    /// List active tunnels
    List,

    /// Create tunnel to peer
    Create { peer_id: String, endpoint: String },

    /// Close tunnel
    Close { peer_id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| if cli.verbose { "debug".into() } else { "info".into() }),
        )
        .init();

    match cli.command {
        Commands::Init { node_type } => {
            info_line("Initializing Helium...");
            init_node(&node_type).await?;
        }

        Commands::Start { daemon } => {
            info_line("Starting Helium node...");
            start_node(daemon).await?;
        }

        Commands::Stop => {
            info_line("Stopping Helium node...");
            stop_node().await?;
        }

        Commands::Advertise { resource_type, capacity, price } => {
            println!(
                "Advertising {} {} at ${}/hour",
                style(capacity.to_string()).bold(),
                resource_type,
                price
            );
            advertise_resources(&resource_type, capacity, price).await?;
        }

        Commands::Request { resource_type, min_capacity, max_price, duration } => {
            println!(
                "Requesting {} {} for {}h (max ${}/hour)",
                style(min_capacity.to_string()).bold(),
                resource_type,
                duration,
                max_price
            );
            request_resources(&resource_type, min_capacity, max_price, duration).await?;
        }

        Commands::Peers => {
            list_peers().await?;
        }

        Commands::Connect { peer_id } => {
            let short = &peer_id[..8.min(peer_id.len())];
            info_line(&format!("Connecting to {}...", short));
            connect_peer(&peer_id).await?;
        }

        Commands::Vm { command } => {
            handle_vm_command(command).await?;
        }

        Commands::Tunnel { command } => {
            handle_tunnel_command(command).await?;
        }

        Commands::Status => {
            show_status().await?;
        }

        Commands::Doctor => {
            run_diagnostics().await?;
        }
    }

    Ok(())
}

async fn init_node(node_type: &str) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    spinner.set_message("Creating configuration...");

    // TODO(S1): create config dir, generate Ed25519 keypair, write config.toml

    spinner.finish_and_clear();
    ok(&format!(
        "Helium {} node initialized. Config: ~/.helium/config.toml",
        node_type
    ));
    Ok(())
}

async fn start_node(daemon: bool) -> Result<()> {
    if daemon {
        info!("Starting in daemon mode");
        // TODO(S2): daemonize process
    }

    let mut node = HeliumNode::new().await?;
    node.start().await?;

    ok("Node is running");
    println!("   Press Ctrl+C to stop");

    tokio::signal::ctrl_c().await?;
    warn_line("Shutting down...");

    Ok(())
}

async fn stop_node() -> Result<()> {
    // TODO(S2): send shutdown signal to daemon
    ok("Node stopped");
    Ok(())
}

async fn advertise_resources(resource_type: &str, _capacity: u32, _price: f64) -> Result<()> {
    validate_resource_type(resource_type)?;
    // TODO(S3): create ResourceOffer, advertise via DHT
    ok("Resource advertised on community DHT");
    Ok(())
}

async fn request_resources(
    resource_type: &str,
    _min_capacity: u32,
    _max_price: f64,
    _duration: u32,
) -> Result<()> {
    validate_resource_type(resource_type)?;
    // TODO(S3): create ResourceRequest, find matches, display results table
    ok("Request published, waiting for matches");
    Ok(())
}

fn validate_resource_type(rt: &str) -> Result<()> {
    match rt {
        "cpu" | "ram" | "gpu" => Ok(()),
        other => Err(anyhow::anyhow!(
            "invalid resource type '{}': expected cpu|ram|gpu",
            other
        )),
    }
}

async fn list_peers() -> Result<()> {
    header("Discovered Peers");
    println!("{:<12} {:<20} {:<15} {:<10}", "Peer ID", "Endpoint", "Resources", "Price/h");
    println!("{}", "-".repeat(60));

    // TODO(S1): query discovery service for live peers

    Ok(())
}

async fn connect_peer(_peer_id: &str) -> Result<()> {
    // TODO(S1): establish WireGuard tunnel to peer
    ok("Tunnel established");
    Ok(())
}

async fn handle_vm_command(command: VmCommands) -> Result<()> {
    match command {
        VmCommands::List => {
            header("Active VMs");
            println!("{:<16} {:<8} {:<8} {:<10}", "VM ID", "vCPUs", "RAM MiB", "State");
            println!("{}", "-".repeat(46));
            // TODO(S2): list VMs from VmManager
        }
        VmCommands::Create { vcpus, memory } => {
            info_line(&format!("Creating VM with {} vCPUs, {} MiB RAM", vcpus, memory));
            // TODO(S2): create VM
        }
        VmCommands::Start { vm_id } => {
            info_line(&format!("Starting VM {}", vm_id));
            // TODO(S2): start VM
        }
        VmCommands::Stop { vm_id } => {
            info_line(&format!("Stopping VM {}", vm_id));
            // TODO(S2): stop VM
        }
        VmCommands::Kill { vm_id } => {
            warn_line(&format!("Force killing VM {}", vm_id));
            // TODO(S2): kill VM
        }
    }
    Ok(())
}

async fn handle_tunnel_command(command: TunnelCommands) -> Result<()> {
    match command {
        TunnelCommands::List => {
            header("Active Tunnels");
            println!("{:<12} {:<22} {:<10}", "Peer ID", "Endpoint", "State");
            println!("{}", "-".repeat(48));
            // TODO(S1): list tunnels from TunnelManager
        }
        TunnelCommands::Create { peer_id, endpoint } => {
            let short = &peer_id[..8.min(peer_id.len())];
            info_line(&format!("Creating tunnel to {} at {}", short, endpoint));
            // TODO(S1): create tunnel
        }
        TunnelCommands::Close { peer_id } => {
            let short = &peer_id[..8.min(peer_id.len())];
            info_line(&format!("Closing tunnel to {}", short));
            // TODO(S1): close tunnel
        }
    }
    Ok(())
}

async fn show_status() -> Result<()> {
    header("Helium Node Status");
    println!();

    println!("{}", style("Node Info:").bold());
    println!("  Peer ID:    (not started)");
    println!("  Status:     stopped");
    println!("  Uptime:     -");
    println!();

    println!("{}", style("Network:").bold());
    println!("  Peers:      0");
    println!("  Tunnels:    0");
    println!();

    println!("{}", style("Resources:").bold());
    println!("  Advertised: 0");
    println!("  Active VMs: 0");

    // TODO(S1): get actual status from node

    Ok(())
}

async fn run_diagnostics() -> Result<()> {
    header("Running Diagnostics");
    println!();

    let checks = vec![
        ("WireGuard kernel module", false),
        ("Firecracker binary", false),
        ("Network connectivity", true),
        ("Port 51820 available", true),
        ("Config directory", true),
    ];

    for (check, status) in checks {
        if status {
            ok(check);
        } else {
            err(check);
        }
    }

    println!();
    // TODO(S1): actually check these things
    warn_line("Some checks failed. Run `helium init` to setup.");

    Ok(())
}
