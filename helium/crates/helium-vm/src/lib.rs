//! Helium VM - Firecracker MicroVM Orchestration
//!
//! Manages Firecracker microVMs for secure, isolated compute
//! execution on provider machines.

pub mod firecracker;
pub mod resource;

use anyhow::Result;
use std::path::PathBuf;

/// VM Manager for Firecracker microVMs
pub struct VmManager {
    firecracker_path: PathBuf,
    kernel_path: PathBuf,
    rootfs_path: PathBuf,
    active_vms: Vec<MicroVm>,
}

/// MicroVM instance
#[derive(Debug)]
pub struct MicroVm {
    pub id: String,
    pub config: VmConfig,
    pub status: VmStatus,
    pub firecracker_pid: Option<u32>,
}

/// VM configuration
#[derive(Debug, Clone)]
pub struct VmConfig {
    pub vcpu_count: u32,
    pub mem_size_mib: u32,
    pub disk_size_gb: u32,
    pub network_namespace: String,
    pub vsock_cid: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmStatus {
    Creating,
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Failed,
}

impl VmManager {
    pub fn new(
        firecracker_path: PathBuf,
        kernel_path: PathBuf,
        rootfs_path: PathBuf,
    ) -> Self {
        Self {
            firecracker_path,
            kernel_path,
            rootfs_path,
            active_vms: Vec::new(),
        }
    }

    /// Create a new microVM with given configuration
    pub async fn create_vm(&self, config: VmConfig) -> Result<MicroVm> {
        let vm_id = format!("helium-{}", uuid::Uuid::new_v4());
        tracing::info!("Creating microVM {} with {} vCPUs, {} MiB RAM",
            vm_id, config.vcpu_count, config.mem_size_mib);

        // TODO:
        // 1. Create jailer environment
        // 2. Configure Firecracker API socket
        // 3. Set boot source (kernel)
        // 4. Set root drive
        // 5. Configure network (tap device)
        // 6. Start VM

        let vm = MicroVm {
            id: vm_id,
            config,
            status: VmStatus::Creating,
            firecracker_pid: None,
        };

        Ok(vm)
    }

    /// Start a microVM
    pub async fn start_vm(&self, vm: &mut MicroVm) -> Result<()> {
        tracing::info!("Starting microVM {}", vm.id);

        // TODO: Launch firecracker process with jailer

        vm.status = VmStatus::Starting;
        Ok(())
    }

    /// Stop a microVM gracefully
    pub async fn stop_vm(&self, vm: &mut MicroVm) -> Result<()> {
        tracing::info!("Stopping microVM {}", vm.id);

        // TODO: Send shutdown signal via API
        // Wait for graceful shutdown, then force if needed

        vm.status = VmStatus::Stopping;
        Ok(())
    }

    /// Force kill a microVM
    pub async fn kill_vm(&self, vm: &mut MicroVm) -> Result<()> {
        tracing::warn!("Force killing microVM {}", vm.id);

        if let Some(pid) = vm.firecracker_pid {
            // Send SIGKILL
            #[cfg(unix)]
            {
                use std::process::Command;
                Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output()?;
            }
        }

        vm.status = VmStatus::Stopped;
        Ok(())
    }

    /// Get VM metrics
    pub async fn get_metrics(&self, vm: &MicroVm) -> Result<VmMetrics> {
        // TODO: Query Firecracker API for metrics
        Ok(VmMetrics {
            cpu_usage_percent: 0.0,
            memory_used_mib: 0,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
        })
    }
}

/// VM performance metrics
#[derive(Debug)]
pub struct VmMetrics {
    pub cpu_usage_percent: f64,
    pub memory_used_mib: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_vm_manager_creation() {
        let manager = VmManager::new(
            PathBuf::from("/usr/bin/firecracker"),
            PathBuf::from("/var/lib/helium/vmlinux"),
            PathBuf::from("/var/lib/helium/rootfs.ext4"),
        );
        assert!(manager.active_vms.is_empty());
    }

    #[test]
    fn test_vm_config() {
        let config = VmConfig {
            vcpu_count: 4,
            mem_size_mib: 8192,
            disk_size_gb: 20,
            network_namespace: "helium0".to_string(),
            vsock_cid: 3,
        };
        assert_eq!(config.vcpu_count, 4);
        assert_eq!(config.mem_size_mib, 8192);
    }
}
