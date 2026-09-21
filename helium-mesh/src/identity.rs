use anyhow::{Context, Result};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, debug};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

const HELIUM_DIR: &str = ".helium";
const IDENTITY_FILE: &str = "identity.json";
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    pub name: Option<String>,
    pub public_key: String,
    pub private_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub listen_address: String,
    pub bootstrap_peers: Vec<String>,
    pub max_peers: usize,
    pub enable_upnp: bool,
    pub wireguard_port: u16,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0:51820".to_string(),
            bootstrap_peers: vec![],
            max_peers: 50,
            enable_upnp: true,
            wireguard_port: 51820,
        }
    }
}

pub struct IdentityManager {
    identity: NodeIdentity,
    config: NodeConfig,
    base_path: PathBuf,
}

impl IdentityManager {
    /// Initialize a new Helium node identity
    pub async fn initialize(name: Option<String>) -> Result<Self> {
        let base_path = Self::helium_dir()?;

        // Create helium directory if it doesn't exist
        tokio::fs::create_dir_all(&base_path).await
            .context("Failed to create helium directory")?;

        // Generate Ed25519 keypair
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let private_key = URL_SAFE_NO_PAD.encode(signing_key.to_bytes());
        let public_key = URL_SAFE_NO_PAD.encode(verifying_key.to_bytes());

        let identity = NodeIdentity {
            name,
            public_key: public_key.clone(),
            private_key: private_key.clone(),
            created_at: chrono::Utc::now(),
        };

        let config = NodeConfig::default();

        let manager = Self {
            identity,
            config,
            base_path: base_path.clone(),
        };

        // Save identity and config
        manager.save().await?;

        info!("Initialized new node with public key: {}", public_key);

        Ok(manager)
    }

    /// Load existing identity
    pub async fn load() -> Result<Self> {
        let base_path = Self::helium_dir()?;

        let identity_path = base_path.join(IDENTITY_FILE);
        let config_path = base_path.join(CONFIG_FILE);

        let identity_json = tokio::fs::read_to_string(&identity_path)
            .await
            .context("Failed to read identity file. Run 'helium init' first.")?;

        let identity: NodeIdentity = serde_json::from_str(&identity_json)
            .context("Failed to parse identity file")?;

        let config = if config_path.exists() {
            let config_str = tokio::fs::read_to_string(&config_path).await?;
            toml::from_str(&config_str).unwrap_or_default()
        } else {
            NodeConfig::default()
        };

        debug!("Loaded identity for node: {:?}", identity.name);

        Ok(Self {
            identity,
            config,
            base_path,
        })
    }

    /// Save identity and config to disk
    async fn save(&self) -> Result<()> {
        let identity_path = self.base_path.join(IDENTITY_FILE);
        let config_path = self.base_path.join(CONFIG_FILE);

        let identity_json = serde_json::to_string_pretty(&self.identity)
            .context("Failed to serialize identity")?;

        let config_toml = toml::to_string_pretty(&self.config)
            .context("Failed to serialize config")?;

        tokio::fs::write(&identity_path, identity_json).await
            .context("Failed to write identity file")?;

        tokio::fs::write(&config_path, config_toml).await
            .context("Failed to write config file")?;

        // Set restrictive permissions (on Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&identity_path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&identity_path, perms)?;
        }

        info!("Saved identity to {}", identity_path.display());

        Ok(())
    }

    /// Get node ID (public key)
    pub fn node_id(&self) -> Result<String> {
        Ok(self.identity.public_key.clone())
    }

    /// Get public key in base58 format (for display)
    pub fn public_key_base58(&self) -> Result<String> {
        // Decode from base64url and re-encode to base58 for shorter display
        let bytes = URL_SAFE_NO_PAD.decode(&self.identity.public_key)?;
        Ok(bs58::encode(bytes).into_string())
    }

    /// Get signing key
    pub fn signing_key(&self) -> Result<SigningKey> {
        let bytes = URL_SAFE_NO_PAD.decode(&self.identity.private_key)?;
        let key_bytes: [u8; 32] = bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid private key length"))?;
        Ok(SigningKey::from_bytes(&key_bytes))
    }

    /// Get verifying key (Phase 2: invitation signature checks on join).
    #[allow(dead_code)]
    pub fn verifying_key(&self) -> Result<VerifyingKey> {
        let bytes = URL_SAFE_NO_PAD.decode(&self.identity.public_key)?;
        let key_bytes: [u8; 32] = bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid public key length"))?;
        Ok(VerifyingKey::from_bytes(&key_bytes)?)
    }

    /// Get config path
    pub fn config_path(&self) -> &Path {
        &self.base_path
    }

    /// Get config (Phase 2: listen address / peer limits).
    #[allow(dead_code)]
    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    /// Get helium directory path
    fn helium_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Failed to determine home directory")?;
        Ok(home.join(HELIUM_DIR))
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Result<Signature> {
        let signing_key = self.signing_key()?;
        Ok(signing_key.sign(message))
    }

    /// Verify a signature (Phase 2: verified when joining with invitation).
    #[allow(dead_code)]
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<bool> {
        let verifying_key = self.verifying_key()?;
        Ok(verifying_key.verify(message, signature).is_ok())
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_identity_lifecycle() {
        // Use temp directory for testing
        // Note: In real tests we'd mock the helium_dir
    }
}
