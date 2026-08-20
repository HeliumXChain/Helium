//! Resource management for VMs
//! 
//! Handles cgroup limits, CPU pinning, and memory allocation

use anyhow::Result;

/// Resource limits for a microVM
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub cpu_shares: u64,
    pub cpu_quota: i64,  // -1 for unlimited
    pub cpu_period: u64,
    pub memory_limit_mib: u64,
    pub memory_swap_limit_mib: u64,
    pub disk_io_weight: u16,
    pub network_bandwidth_mbps: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_shares: 1024,
            cpu_quota: -1,
            cpu_period: 100000,
            memory_limit_mib: 1024,
            memory_swap_limit_mib: 1024,
            disk_io_weight: 100,
            network_bandwidth_mbps: 1000,
        }
    }
}

/// Resource controller using cgroups
pub struct ResourceController {
    cgroup_path: String,
}

impl ResourceController {
    pub fn new(vm_id: &str) -> Self {
        Self {
            cgroup_path: format!("/sys/fs/cgroup/helium/{}", vm_id),
        }
    }
    
    /// Create cgroup for VM
    pub async fn create_cgroup(&self) -> Result<()> {
        tracing::info!("Creating cgroup at {}", self.cgroup_path);
        
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            use std::process::Command;
            
            // Create cgroup v2 directory
            fs::create_dir_all(&self.cgroup_path)?;
            
            // Enable controllers
            Command::new("sh")
                .args(["-c", &format!(
                    "echo '+cpu +memory +io' > /sys/fs/cgroup/cgroup.subtree_control"
                )])
                .output()?;
        }
        
        Ok(())
    }
    
    /// Apply resource limits
    pub async fn apply_limits(&self, limits: &ResourceLimits) -> Result<()> {
        tracing::info!("Applying resource limits to {}", self.cgroup_path);
        
        #[cfg(target_os = "linux")]
        {
            use std::fs::write;
            
            // CPU limits
            write(
                format!("{}/cpu.max", self.cgroup_path),
                format!("{} {}", limits.cpu_quota, limits.cpu_period),
            )?;
            
            write(
                format!("{}/cpu.weight", self.cgroup_path),
                limits.cpu_shares.to_string(),
            )?;
            
            // Memory limits
            write(
                format!("{}/memory.max", self.cgroup_path),
                (limits.memory_limit_mib * 1024 * 1024).to_string(),
            )?;
            
            write(
                format!("{}/memory.swap.max", self.cgroup_path),
                (limits.memory_swap_limit_mib * 1024 * 1024).to_string(),
            )?;
        }
        
        Ok(())
    }
    
    /// Assign Firecracker process to cgroup
    pub async fn assign_process(&self, pid: u32) -> Result<()> {
        tracing::info!("Assigning PID {} to cgroup {}", pid, self.cgroup_path);
        
        #[cfg(target_os = "linux")]
        {
            use std::fs::write;
            write(
                format!("{}/cgroup.procs", self.cgroup_path),
                pid.to_string(),
            )?;
        }
        
        Ok(())
    }
    
    /// Get current resource usage
    pub async fn get_usage(&self) -> Result<ResourceUsage> {
        #[cfg(target_os = "linux")]
        {
            use std::fs::read_to_string;
            
            let cpu_stat = read_to_string(format!("{}/cpu.stat", self.cgroup_path))?;
            let memory_current = read_to_string(format!("{}/memory.current", self.cgroup_path))?
                .trim()
                .parse::<u64>()?;
            
            Ok(ResourceUsage {
                cpu_usage_nanos: Self::parse_cpu_usage(&cpu_stat),
                memory_bytes: memory_current,
            })
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            Ok(ResourceUsage {
                cpu_usage_nanos: 0,
                memory_bytes: 0,
            })
        }
    }
    
    fn parse_cpu_usage(cpu_stat: &str) -> u64 {
        // Parse "usage_usec 12345678" from cpu.stat
        cpu_stat
            .lines()
            .find(|line| line.starts_with("usage_usec"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(0)
            * 1000 // Convert to nanoseconds
    }
    
    /// Remove cgroup
    pub async fn remove_cgroup(&self) -> Result<()> {
        tracing::info!("Removing cgroup {}", self.cgroup_path);
        
        #[cfg(target_os = "linux")]
        {
            use std::fs::remove_dir;
            remove_dir(&self.cgroup_path)?;
        }
        
        Ok(())
    }
}

/// Current resource usage
#[derive(Debug)]
pub struct ResourceUsage {
    pub cpu_usage_nanos: u64,
    pub memory_bytes: u64,
}
