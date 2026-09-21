//! WireGuard tunnel management via the system `wg` / `ip` tools.
//!
//! No extra dependencies (boringtun/wireguard-rs = Phase 2). All mutating
//! operations require root — error messages say so explicitly.

use anyhow::{bail, Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};

pub struct WireGuardInterface {
    name: String,
}

#[derive(Debug, Clone)]
pub struct PeerConfig {
    pub public_key: String,
    pub endpoint: Option<String>,
    pub allowed_ips: Vec<String>,
    pub keepalive: Option<u16>,
}

/// Run a command, retrying once via non-interactive sudo on permission
/// errors (works with the NOPASSWD rule installed by `install.sh --provider`).
fn run(cmd: &str, args: &[&str]) -> Result<String> {
    match run_inner(cmd, args) {
        Ok(out) => Ok(out),
        Err(e) if e.to_string().contains("needs root") => {
            let mut sudo_args = vec!["-n", cmd];
            sudo_args.extend(args.iter().copied());
            run_inner("sudo", &sudo_args)
        }
        Err(e) => Err(e),
    }
}

fn run_inner(cmd: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("cannot execute {cmd} (wireguard-tools installed?)"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        if err.contains("Operation not permitted") || err.contains("Permission denied") {
            bail!("{cmd} needs root — rerun with sudo");
        }
        bail!("{cmd} {} failed: {}", args.join(" "), err.trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Generate a fresh keypair using `wg genkey` / `wg pubkey`.
pub fn genkeypair() -> Result<(String, String)> {
    let privkey = run("wg", &["genkey"])?;
    let pubkey = public_key_of(&privkey)?;
    Ok((privkey, pubkey))
}

/// Derive the public key from a private key (no root needed).
pub fn public_key_of(private_key: &str) -> Result<String> {
    let mut child = Command::new("wg")
        .arg("pubkey")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("cannot execute wg pubkey")?;
    child
        .stdin
        .as_mut()
        .context("cannot pipe to wg")?
        .write_all(private_key.trim().as_bytes())?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        bail!("wg pubkey failed (invalid key?)");
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Render a wg-quick-style config (pure, unit-tested).
pub fn render_config(
    private_key: &str,
    address: &str,
    listen_port: Option<u16>,
    peers: &[PeerConfig],
) -> String {
    let mut s = String::from("[Interface]\n");
    s.push_str(&format!("PrivateKey = {private_key}\n"));
    s.push_str(&format!("Address = {address}\n"));
    if let Some(port) = listen_port {
        s.push_str(&format!("ListenPort = {port}\n"));
    }
    for p in peers {
        s.push_str("\n[Peer]\n");
        s.push_str(&format!("PublicKey = {}\n", p.public_key));
        if let Some(ep) = &p.endpoint {
            s.push_str(&format!("Endpoint = {ep}\n"));
        }
        s.push_str(&format!("AllowedIPs = {}\n", p.allowed_ips.join(", ")));
        if let Some(ka) = p.keepalive {
            s.push_str(&format!("PersistentKeepalive = {ka}\n"));
        }
    }
    s
}

impl WireGuardInterface {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn exists(&self) -> bool {
        Command::new("ip")
            .args(["link", "show", &self.name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Create (or recreate) the interface. Needs root.
    pub fn up(&self, address: &str, listen_port: u16, private_key: &str) -> Result<()> {
        if private_key.trim().is_empty() {
            bail!("empty private key");
        }
        if self.exists() {
            self.down()?;
        }
        run("ip", &["link", "add", "dev", &self.name, "type", "wireguard"])?;
        let set_status = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "echo {} | wg set {} listen-port {} private-key /dev/stdin",
                private_key.trim(),
                self.name,
                listen_port
            ))
            .status()?;
        if !set_status.success() {
            let _ = self.down();
            bail!("wg set failed (needs root?)");
        }
        run("ip", &["address", "add", address, "dev", &self.name])?;
        run("ip", &["link", "set", "up", "dev", &self.name])?;
        Ok(())
    }

    /// Delete the interface. Needs root.
    pub fn down(&self) -> Result<()> {
        if self.exists() {
            run("ip", &["link", "del", &self.name])?;
        }
        Ok(())
    }

    /// Add or update a peer. Needs root.
    pub fn add_peer(&self, peer: &PeerConfig) -> Result<()> {
        if peer.public_key.trim().is_empty() {
            bail!("empty peer public key");
        }
        let mut args = vec![
            "set".to_string(),
            self.name.clone(),
            "peer".to_string(),
            peer.public_key.trim().to_string(),
        ];
        if let Some(ep) = &peer.endpoint {
            args.push("endpoint".to_string());
            args.push(ep.clone());
        }
        args.push("allowed-ips".to_string());
        args.push(if peer.allowed_ips.is_empty() {
            "(none)".to_string()
        } else {
            peer.allowed_ips.join(",")
        });
        if let Some(ka) = peer.keepalive {
            args.push("persistent-keepalive".to_string());
            args.push(ka.to_string());
        }
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        run("wg", &refs)?;
        Ok(())
    }

    /// Remove a peer. Needs root.
    pub fn remove_peer(&self, public_key: &str) -> Result<()> {
        run("wg", &["set", &self.name, "peer", public_key, "remove"])?;
        Ok(())
    }

    /// Full `wg show` dump (works unprivileged for reading on most setups).
    pub fn show(&self) -> Result<String> {
        run("wg", &["show", &self.name])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keypair_roundtrip() -> Result<()> {
        let (privkey, pubkey) = genkeypair()?;
        assert_eq!(pubkey, public_key_of(&privkey)?);
        assert!(privkey.len() >= 40 && pubkey.len() >= 40);
        Ok(())
    }

    #[test]
    fn render_provider_config() {
        let conf = render_config(
            "PRIV",
            "10.0.0.1/24",
            Some(51820),
            &[PeerConfig {
                public_key: "PUB".to_string(),
                endpoint: None,
                allowed_ips: vec!["10.0.0.2/32".to_string()],
                keepalive: None,
            }],
        );
        assert!(conf.contains("[Interface]"));
        assert!(conf.contains("ListenPort = 51820"));
        assert!(conf.contains("PublicKey = PUB"));
        assert!(conf.contains("AllowedIPs = 10.0.0.2/32"));
    }

    #[test]
    fn up_rejects_empty_key() {
        let wg = WireGuardInterface::new("helium-test-nonexistent");
        assert!(wg.up("10.99.0.1/24", 56789, "   ").is_err());
        assert!(wg
            .add_peer(&PeerConfig {
                public_key: String::new(),
                endpoint: None,
                allowed_ips: vec![],
                keepalive: None,
            })
            .is_err());
    }
}
