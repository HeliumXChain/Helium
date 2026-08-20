//! Firecracker-specific implementation
//! 
//! Direct API client for Firecracker microVM management

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Firecracker API client
pub struct FirecrackerClient {
    client: Client,
    api_url: String,
}

impl FirecrackerClient {
    pub fn new(api_socket: &str) -> Self {
        // Firecracker uses Unix sockets, but we communicate via HTTP over the socket
        let api_url = format!("http+unix://{}", api_socket.replace('/', "%2F"));
        
        Self {
            client: Client::new(),
            api_url,
        }
    }
    
    /// Configure boot source (kernel)
    pub async fn put_boot_source(&self, kernel_path: &str) -> Result<()> {
        let boot_source = BootSource {
            kernel_image_path: kernel_path.to_string(),
            boot_args: Some("console=ttyS0 reboot=k panic=1 pci=off".to_string()),
        };
        
        tracing::info!("Configuring boot source: {}", kernel_path);
        // TODO: HTTP PUT to /boot-source
        Ok(())
    }
    
    /// Configure root drive
    pub async fn put_drive(&self, drive_id: &str, path: &str) -> Result<()> {
        let drive = Drive {
            drive_id: drive_id.to_string(),
            path_on_host: path.to_string(),
            is_root_device: drive_id == "rootfs",
            is_read_only: false,
        };
        
        tracing::info!("Configuring drive {}: {}", drive_id, path);
        // TODO: HTTP PUT to /drives/{drive_id}
        Ok(())
    }
    
    /// Configure network interface
    pub async fn put_network_interface(&self, iface_id: &str, tap_dev: &str) -> Result<()> {
        let network = NetworkInterface {
            iface_id: iface_id.to_string(),
            host_dev_name: tap_dev.to_string(),
            guest_mac: None,
        };
        
        tracing::info!("Configuring network interface {}: {}", iface_id, tap_dev);
        // TODO: HTTP PUT to /network-interfaces/{iface_id}
        Ok(())
    }
    
    /// Configure machine (vCPUs, memory)
    pub async fn put_machine_config(&self, vcpu_count: u32, mem_size_mib: u32) -> Result<()> {
        let config = MachineConfig {
            vcpu_count,
            mem_size_mib,
            smt: false,
            track_dirty_pages: false,
        };
        
        tracing::info!("Configuring machine: {} vCPUs, {} MiB", vcpu_count, mem_size_mib);
        // TODO: HTTP PUT to /machine-config
        Ok(())
    }
    
    /// Start the VM
    pub async fn create_instance(&self) -> Result<()> {
        tracing::info!("Starting microVM instance");
        // TODO: HTTP PUT to /actions with ActionType::InstanceCreate
        Ok(())
    }
    
    /// Stop the VM
    pub async fn stop_instance(&self) -> Result<()> {
        tracing::info!("Stopping microVM instance");
        // TODO: HTTP PUT to /actions with ActionType::SendCtrlAltDel
        // Then force shutdown after timeout if needed
        Ok(())
    }
    
    /// Get instance info
    pub async fn get_instance_info(&self) -> Result<InstanceInfo> {
        // TODO: HTTP GET to /
        Ok(InstanceInfo {
            id: "test".to_string(),
            state: "Running".to_string(),
        })
    }
}

// Firecracker API types

#[derive(Debug, Serialize, Deserialize)]
struct BootSource {
    kernel_image_path: String,
    boot_args: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Drive {
    drive_id: String,
    path_on_host: String,
    is_root_device: bool,
    is_read_only: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct NetworkInterface {
    iface_id: String,
    host_dev_name: String,
    guest_mac: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MachineConfig {
    vcpu_count: u32,
    mem_size_mib: u32,
    smt: bool,
    track_dirty_pages: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Action {
    action_type: String,
}

#[derive(Debug, Deserialize)]
struct InstanceInfo {
    id: String,
    state: String,
}

/// Setup helper for Firecracker binaries
pub struct FirecrackerSetup;

impl FirecrackerSetup {
    /// Download and setup Firecracker binary
    pub async fn setup(version: &str, install_dir: &str) -> Result<PathBuf> {
        tracing::info!("Setting up Firecracker v{} in {}", version, install_dir);
        
        // TODO:
        // 1. Download from GitHub releases
        // 2. Verify checksum
        // 3. Make executable
        // 4. Download kernel and rootfs image
        
        let path = PathBuf::from(install_dir).join("firecracker");
        Ok(path)
    }
    
    /// Check if firecracker is installed
    pub fn is_installed() -> bool {
        which::which("firecracker").is_ok()
    }
}
