//! WireGuard interface management

use anyhow::{anyhow, Result};
use std::process::Command;

/// WireGuard interface configuration
#[derive(Debug, Clone)]
pub struct WireGuardConfig {
    pub interface_name: String,
    pub private_key: String,
    pub address: String,
    pub listen_port: u16,
    pub dns: Option<String>,
}

/// WireGuard interface (platform-specific)
pub struct WireGuardInterface {
    config: WireGuardConfig,
}

impl WireGuardInterface {
    pub fn new(config: WireGuardConfig) -> Self {
        Self { config }
    }

    /// Create interface (platform-specific)
    pub async fn create(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        return self.create_windows().await;

        #[cfg(target_os = "linux")]
        return self.create_linux().await;

        #[cfg(target_os = "macos")]
        return self.create_macos().await;

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        return Err(anyhow!("Unsupported platform"));
    }

    #[cfg(target_os = "windows")]
    async fn create_windows(&self) -> Result<()> {
        // Windows: Use wireguard.exe or native APIs
        tracing::info!("Creating WireGuard interface {} on Windows", self.config.interface_name);
        // TODO: Implement Windows-specific setup
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn create_linux(&self) -> Result<()> {
        // Linux: Use ip and wg commands
        tracing::info!("Creating WireGuard interface {} on Linux", self.config.interface_name);

        // ip link add dev wg0 type wireguard
        Command::new("ip")
            .args(["link", "add", "dev", &self.config.interface_name, "type", "wireguard"])
            .output()?;

        // ip address add dev wg0 10.0.0.1/24
        Command::new("ip")
            .args(["address", "add", "dev", &self.config.interface_name, &self.config.address])
            .output()?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn create_macos(&self) -> Result<()> {
        // macOS: Use wireguard-go or native NetworkExtension
        tracing::info!("Creating WireGuard interface {} on macOS", self.config.interface_name);
        // TODO: Implement macOS-specific setup
        Ok(())
    }

    /// Add a peer to the interface
    pub async fn add_peer(&self, public_key: &str, endpoint: &str, allowed_ips: &[String]) -> Result<()> {
        tracing::info!("Adding peer {} with endpoint {}", &public_key[..8], endpoint);

        let allowed = allowed_ips.join(",");

        #[cfg(target_os = "linux")]
        {
            Command::new("wg")
                .args([
                    "set", &self.config.interface_name,
                    "peer", public_key,
                    "endpoint", endpoint,
                    "allowed-ips", &allowed,
                ])
                .output()?;
        }

        Ok(())
    }

    /// Remove peer
    pub async fn remove_peer(&self, public_key: &str) -> Result<()> {
        tracing::info!("Removing peer {}", &public_key[..8]);

        #[cfg(target_os = "linux")]
        {
            Command::new("wg")
                .args(["set", &self.config.interface_name, "peer", public_key, "remove"])
                .output()?;
        }

        Ok(())
    }

    /// Bring interface up
    pub async fn up(&self) -> Result<()> {
        tracing::info!("Bringing up interface {}", self.config.interface_name);

        #[cfg(target_os = "linux")]
        {
            Command::new("ip")
                .args(["link", "set", "up", "dev", &self.config.interface_name])
                .output()?;
        }

        Ok(())
    }

    /// Bring interface down
    pub async fn down(&self) -> Result<()> {
        tracing::info!("Bringing down interface {}", self.config.interface_name);

        #[cfg(target_os = "linux")]
        {
            Command::new("ip")
                .args(["link", "set", "down", "dev", &self.config.interface_name])
                .output()?;
        }

        Ok(())
    }
}

/// Generate WireGuard keypair
pub fn generate_keypair() -> Result<(String, String)> {
    // Use wg genkey | wg pubkey
    let private_key = Command::new("wg")
        .args(["genkey"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

    let public_key = Command::new("wg")
        .args(["pubkey"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?
        .stdin
        .take()
        .ok_or_else(|| anyhow!("Failed to get stdin"));

    // TODO: Complete keypair generation
    Ok((private_key, "public_key_placeholder".to_string()))
}
