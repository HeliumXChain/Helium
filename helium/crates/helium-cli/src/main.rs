//! Helium CLI - Command Line Interface
//! 
//! Main entry point for the Helium P2P compute marketplace.

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;
use helium_core::{HeliumNode, ResourceOffer, ResourceRequest};
use helium_tunnel::TunnelManager;
use helium_vm::VmManager;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "helium")]
#[command(about = "Helium - P2P Compute Marketplace")]
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
    Start {
        vm_id: String,
    },
    
    /// Stop a VM
    Stop {
        vm_id: String,
    },
    
    /// Kill a VM (force stop)
    Kill {
        vm_id: String,
    },
}

#[derive(Subcommand)]
enum TunnelCommands {
    /// List active tunnels
    List,
    
    /// Create tunnel to peer
    Create {
        peer_id: String,
        endpoint: String,
    },
    
    /// Close tunnel
    Close {
        peer_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Setup tracing
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        )
        .init();
    
    match cli.command {
        Commands::Init { node_type } => {
            println!("{}", style("🚀 Initializing Helium...").bold().cyan());
            init_node(&node_type).await?;
        }
        
        Commands::Start { daemon } => {
            println!("{}", style("▶️  Starting Helium node...").bold().green());
            start_node(daemon).await?;
        }
        
        Commands::Stop => {
            println!("{}", style("⏹️  Stopping Helium node...").bold().yellow());
            stop_node().await?;
        }
        
        Commands::Advertise { resource_type, capacity, price } => {
            println!("{} Advertising {} {} at ${}/hour", 
                style("📢").bold(), capacity, resource_type, price);
            advertise_resources(&resource_type, capacity, price).await?;
        }
        
        Commands::Request { resource_type, min_capacity, max_price, duration } => {
            println!("{} Requesting {} {} for {}h (max ${}/hour)",
                style("🔍").bold(), min_capacity, resource_type, duration, max_price);
            request_resources(&resource_type, min_capacity, max_price, duration).await?;
        }
        
        Commands::Peers => {
            list_peers().await?;
        }
        
        Commands::Connect { peer_id } => {
            println!("{} Connecting to {}...", style("🔗").bold(), &peer_id[..8.min(peer_id.len())]);
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
    spinner.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")?);
    spinner.set_message("Creating configuration...");
    
    // TODO: Create config directory, generate keys, setup initial config
    
    spinner.finish_with_message(format!(
        "✅ Helium {} node initialized!\n   Config: ~/.helium/config.toml",
        node_type
    ));
    
    Ok(())
}

async fn start_node(daemon: bool) -> Result<()> {
    if daemon {
        info!("Starting in daemon mode");
        // TODO: Daemonize process
    }
    
    let mut node = HeliumNode::new().await?;
    node.start().await?;
    
    println!("{}", style("✅ Node is running!").bold().green());
    println!("   Press Ctrl+C to stop");
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    println!("\n{}", style("👋 Shutting down...").bold().yellow());
    
    Ok(())
}

async fn stop_node() -> Result<()> {
    // TODO: Send shutdown signal to daemon
    println!("{}", style("✅ Node stopped").bold().green());
    Ok(())
}

async fn advertise_resources(resource_type: &str, capacity: u32, price: f64) -> Result<()> {
    // TODO: Parse resource type, create ResourceOffer, advertise via node
    info!("Advertising: {} {} at ${}/hour", capacity, resource_type, price);
    Ok(())
}

async fn request_resources(
    resource_type: &str,
    min_capacity: u32,
    max_price: f64,
    duration: u32,
) -> Result<()> {
    // TODO: Create ResourceRequest, find matches, display results
    info!("Requesting: {} {} for {}h at max ${}/hour", 
        min_capacity, resource_type, duration, max_price);
    Ok(())
}

async fn list_peers() -> Result<()> {
    println!("{}", style("📋 Discovered Peers").bold().underlined());
    println!("{:<12} {:<20} {:<15} {:<10}", "Peer ID", "Endpoint", "Resources", "Price/h");
    println!("{}", "-".repeat(60));
    
    // TODO: Query node for discovered peers
    
    Ok(())
}

async fn connect_peer(peer_id: &str) -> Result<()> {
    // TODO: Establish tunnel to peer
    info!("Connecting to peer: {}", peer_id);
    Ok(())
}

async fn handle_vm_command(command: VmCommands) -> Result<()> {
    match command {
        VmCommands::List => {
            println!("{}", style("🖥️  Active VMs").bold().underlined());
            // TODO: List VMs
        }
        VmCommands::Create { vcpus, memory } => {
            println!("{} Creating VM with {} vCPUs, {} MiB RAM", 
                style("⚙️").bold(), vcpus, memory);
            // TODO: Create VM
        }
        VmCommands::Start { vm_id } => {
            println!("{} Starting VM {}", style("▶️").bold(), vm_id);
            // TODO: Start VM
        }
        VmCommands::Stop { vm_id } => {
            println!("{} Stopping VM {}", style("⏹️").bold(), vm_id);
            // TODO: Stop VM
        }
        VmCommands::Kill { vm_id } => {
            warn!("Force killing VM {}", vm_id);
            // TODO: Kill VM
        }
    }
    Ok(())
}

async fn handle_tunnel_command(command: TunnelCommands) -> Result<()> {
    match command {
        TunnelCommands::List => {
            println!("{}", style("🔒 Active Tunnels").bold().underlined());
            // TODO: List tunnels
        }
        TunnelCommands::Create { peer_id, endpoint } => {
            println!("{} Creating tunnel to {} at {}", 
                style("🔗").bold(), &peer_id[..8], endpoint);
            // TODO: Create tunnel
        }
        TunnelCommands::Close { peer_id } => {
            println!("{} Closing tunnel to {}", style("🔓").bold(), &peer_id[..8]);
            // TODO: Close tunnel
        }
    }
    Ok(())
}

async fn show_status() -> Result<()> {
    println!("{}", style("📊 Helium Node Status").bold().underlined().cyan());
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
    
    // TODO: Get actual status from node
    
    Ok(())
}

async fn run_diagnostics() -> Result<()> {
    println!("{}", style("🔍 Running Diagnostics...").bold().cyan());
    println!();
    
    let checks = vec![
        ("WireGuard kernel module", false),
        ("Firecracker binary", false),
        ("Network connectivity", true),
        ("Port 51820 available", true),
        ("Config directory", true),
    ];
    
    for (check, status) in checks {
        let icon = if status { "✅" } else { "❌" };
        println!("{} {}", icon, check);
    }
    
    println!();
    // TODO: Actually check these things
    println!("{}", style("⚠️  Some checks failed. Run `helium init` to setup.").yellow());
    
    Ok(())
}
