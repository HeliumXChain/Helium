//! Phase 2 scaffolding — datasets/chunks, pas encore branche aux commandes.
#![allow(dead_code)]

use anyhow::{Context, Result};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::RwLock;
use tracing::{info, debug};

const STORAGE_FILE: &str = "storage_manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub chunks: Vec<Chunk>,
    pub replicas: Vec<String>, // Node IDs that have a replica
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub size_bytes: u64,
    pub hash: String,
    pub locations: Vec<String>, // Node IDs where this chunk is stored
    #[serde(default)]
    pub source_path: String, // local file backing this chunk (serve)
    #[serde(default)]
    pub offset: u64, // byte offset of this chunk in source_path
}

pub struct StorageManager {
    datasets: RwLock<HashMap<String, Dataset>>,
    base_path: std::path::PathBuf,
}

impl StorageManager {
    pub async fn new() -> Result<Self> {
        let base_path = Self::helium_dir()?;
        Self::new_in(base_path).await
    }

    /// Constructor with explicit dir (tests, custom homes).
    pub async fn new_in(base_path: std::path::PathBuf) -> Result<Self> {
        tokio::fs::create_dir_all(&base_path).await.ok();
        let datasets = if let Ok(data) = tokio::fs::read_to_string(base_path.join(STORAGE_FILE)).await {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self {
            datasets: RwLock::new(datasets),
            base_path,
        })
    }

    pub async fn register_dataset(&self, name: String, size_bytes: u64) -> Result<Dataset> {
        let id = uuid::Uuid::new_v4().to_string();

        // Calculate chunks (256MB default chunk size)
        let chunk_size = 256 * 1024 * 1024; // 256MB
        let num_chunks = (size_bytes + chunk_size - 1) / chunk_size;

        let mut chunks = vec![];
        for i in 0..num_chunks {
            let chunk_id = format!("{}-{}", id, i);
            chunks.push(Chunk {
                id: chunk_id,
                size_bytes: std::cmp::min(chunk_size, size_bytes - i * chunk_size),
                hash: String::new(), // Will be computed during upload
                locations: vec![],
                source_path: String::new(),
                offset: i * chunk_size,
            });
        }

        let dataset = Dataset {
            id: id.clone(),
            name,
            size_bytes,
            chunks,
            replicas: vec![],
            created_at: chrono::Utc::now(),
        };

        let mut datasets = self.datasets.write().await;
        datasets.insert(id.clone(), dataset.clone());

        self.save().await?;

        info!("Registered dataset {} ({})", id, dataset.name);

        Ok(dataset)
    }

    pub async fn list_datasets(&self) -> Vec<Dataset> {
        let datasets = self.datasets.read().await;
        datasets.values().cloned().collect()
    }

    /// Register a real file: 256MB chunks with SHA-256, manifest persisted.
    /// Records where the bytes live (this node); fetching from peers = next step.
    pub async fn register_file(&self, name: String, path: &std::path::Path) -> Result<Dataset> {
        let mut file = std::fs::File::open(path)
            .with_context(|| format!("cannot open {}", path.display()))?;
        let size_bytes = file.metadata()?.len();
        const CHUNK: u64 = 256 * 1024 * 1024;

        let id = uuid::Uuid::new_v4().to_string();
        let mut chunks = vec![];
        let mut buf = vec![0u8; 4 * 1024 * 1024];
        let mut index = 0u64;
        let mut chunk_hasher = Sha256::new();
        let mut chunk_len = 0u64;
        let mut chunk_offset = 0u64;
        let mut file_offset = 0u64;

        let source = path.to_string_lossy().to_string();
        let flush_chunk =
            |index: u64, hash: String, len: u64, offset: u64, chunks: &mut Vec<Chunk>| {
                chunks.push(Chunk {
                    id: format!("{id}-{index}"),
                    size_bytes: len,
                    hash,
                    locations: vec![],
                    source_path: source.clone(),
                    offset,
                });
            };

        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            let mut slice = &buf[..n];
            while !slice.is_empty() {
                let room = (CHUNK - chunk_len) as usize;
                let take = room.min(slice.len());
                chunk_hasher.update(&slice[..take]);
                chunk_len += take as u64;
                file_offset += take as u64;
                slice = &slice[take..];
                if chunk_len == CHUNK {
                    let hash = format!("{:x}", chunk_hasher.finalize_reset());
                    flush_chunk(index, hash, chunk_len, chunk_offset, &mut chunks);
                    index += 1;
                    chunk_len = 0;
                    chunk_offset = file_offset;
                }
            }
        }
        if chunk_len > 0 || chunks.is_empty() {
            let hash = format!("{:x}", chunk_hasher.finalize());
            flush_chunk(index, hash, chunk_len, chunk_offset, &mut chunks);
        }

        let dataset = Dataset {
            id: id.clone(),
            name: name.clone(),
            size_bytes,
            chunks,
            replicas: vec![],
            created_at: chrono::Utc::now(),
        };

        let mut datasets = self.datasets.write().await;
        datasets.insert(id.clone(), dataset.clone());
        drop(datasets);
        self.save().await?;

        info!(
            "Registered file {} as dataset {} ({} bytes, {} chunks)",
            path.display(),
            id,
            size_bytes,
            dataset.chunks.len()
        );

        Ok(dataset)
    }

    pub async fn get_dataset(&self, id: &str) -> Option<Dataset> {
        let datasets = self.datasets.read().await;
        datasets.get(id).cloned()
    }

    pub async fn update_chunk_location(&self, dataset_id: &str, chunk_id: &str, node_id: String) -> Result<()> {
        let mut datasets = self.datasets.write().await;

        if let Some(dataset) = datasets.get_mut(dataset_id) {
            if let Some(chunk) = dataset.chunks.iter_mut().find(|c| c.id == chunk_id) {
                if !chunk.locations.contains(&node_id) {
                    chunk.locations.push(node_id.clone());
                    debug!("Added location {} for chunk {}", node_id, chunk_id);
                }
            }
            self.save().await?;
        }

        Ok(())
    }

    /// Read one chunk's bytes from its backing file (hash-verified by caller).
    pub async fn read_chunk_bytes(&self, chunk_id: &str) -> Result<Vec<u8>> {
        let chunk = {
            let datasets = self.datasets.read().await;
            datasets
                .values()
                .flat_map(|d| &d.chunks)
                .find(|c| c.id == chunk_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("unknown chunk {chunk_id}"))?
        };
        if chunk.source_path.is_empty() {
            anyhow::bail!("chunk {chunk_id} has no local source file");
        }
        let mut f = tokio::fs::File::open(&chunk.source_path)
            .await
            .with_context(|| format!("cannot open {}", chunk.source_path))?;
        f.seek(std::io::SeekFrom::Start(chunk.offset)).await?;
        let mut buf = vec![0u8; chunk.size_bytes as usize];
        f.read_exact(&mut buf).await?;
        Ok(buf)
    }

    /// Serve manifests + chunks over HTTP until aborted.
    pub async fn serve(self: Arc<Self>, port: u16) -> Result<()> {
        let app = axum::Router::new()
            .route("/manifests", axum::routing::get(h_manifests))
            .route("/manifests/:id", axum::routing::get(h_manifest))
            .route("/chunks/:id", axum::routing::get(h_chunk))
            .with_state(self);
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
        info!("storage serving on port {port}");
        axum::serve(listener, app)
            .await
            .context("storage server failed")?;
        Ok(())
    }

    /// Fetch a dataset from a peer (`base_url` like http://10.0.0.x:8788):
    /// manifest first, then every chunk with SHA-256 verification.
    /// `token` is the peer's API token (empty = no auth header).
    pub async fn fetch(
        base_url: &str,
        dataset_id: &str,
        out_dir: &std::path::Path,
        token: &str,
    ) -> Result<std::path::PathBuf> {
        use tokio::io::AsyncWriteExt;

        let base = base_url.trim_end_matches('/');
        let client = reqwest::Client::new();
        let authed = |b: reqwest::RequestBuilder| {
            if token.is_empty() {
                b
            } else {
                b.bearer_auth(token)
            }
        };
        let ds: Dataset = authed(client.get(format!("{base}/manifests/{dataset_id}")))
            .send()
            .await
            .context("cannot reach peer manifests")?
            .error_for_status()
            .context("peer has no such dataset (or bad token)")?
            .json()
            .await
            .context("bad manifest JSON")?;
        tokio::fs::create_dir_all(out_dir).await.ok();
        let dest = out_dir.join(&ds.name);
        let mut out = tokio::fs::File::create(&dest).await?;
        for chunk in &ds.chunks {
            let bytes = authed(client.get(format!("{base}/chunks/{}", chunk.id)))
                .send()
                .await
                .context("chunk request failed")?
                .error_for_status()
                .context("peer missing chunk")?
                .bytes()
                .await?;
            let mut h = Sha256::new();
            h.update(&bytes);
            let digest = format!("{:x}", h.finalize());
            if digest != chunk.hash {
                anyhow::bail!(
                    "hash mismatch for chunk {}: got {digest}, want {}",
                    chunk.id,
                    chunk.hash
                );
            }
            out.write_all(&bytes).await?;
            info!("fetched chunk {} ({} bytes)", chunk.id, bytes.len());
        }
        out.flush().await?;
        Ok(dest)
    }

    async fn save(&self) -> Result<()> {
        let datasets = self.datasets.read().await;
        let data = serde_json::to_string_pretty(&*datasets)
            .context("Failed to serialize datasets")?;

        tokio::fs::write(self.base_path.join(STORAGE_FILE), data).await
            .context("Failed to write storage manifest")?;

        Ok(())
    }

    fn helium_dir() -> Result<std::path::PathBuf> {
        let home = dirs::home_dir()
            .context("Failed to determine home directory")?;
        Ok(home.join(".helium"))
    }
}

async fn h_manifests(
    State(mgr): State<Arc<StorageManager>>,
    headers: axum::http::HeaderMap,
) -> Result<axum::Json<Vec<Dataset>>, StatusCode> {
    if !crate::identity::authorized(&headers, &crate::identity::api_token()) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(axum::Json(mgr.list_datasets().await))
}

async fn h_manifest(
    State(mgr): State<Arc<StorageManager>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<axum::Json<Dataset>, StatusCode> {
    if !crate::identity::authorized(&headers, &crate::identity::api_token()) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    mgr.get_dataset(&id).await.map(axum::Json).ok_or(StatusCode::NOT_FOUND)
}

async fn h_chunk(
    State(mgr): State<Arc<StorageManager>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Vec<u8>, StatusCode> {
    if !crate::identity::authorized(&headers, &crate::identity::api_token()) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    mgr.read_chunk_bytes(&id).await.map_err(|_| StatusCode::NOT_FOUND)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_small_file() -> Result<()> {
        let dir = tempfile::TempDir::new()?;
        let mgr = StorageManager::new_in(dir.path().to_path_buf()).await?;
        let fp = dir.path().join("data.bin");
        let payload = vec![0xABu8; 5 * 1024 * 1024];
        std::fs::write(&fp, &payload)?;
        let ds = mgr.register_file("data".to_string(), &fp).await?;
        assert_eq!(ds.size_bytes, payload.len() as u64);
        assert_eq!(ds.chunks.len(), 1);
        let mut h = Sha256::new();
        h.update(&payload);
        assert_eq!(ds.chunks[0].hash, format!("{:x}", h.finalize()));
        assert_eq!(mgr.list_datasets().await.len(), 1);
        assert!(mgr.get_dataset(&ds.id).await.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn register_empty_file() -> Result<()> {
        let dir = tempfile::TempDir::new()?;
        let mgr = StorageManager::new_in(dir.path().to_path_buf()).await?;
        let fp = dir.path().join("empty.bin");
        std::fs::write(&fp, b"")?;
        let ds = mgr.register_file("empty".to_string(), &fp).await?;
        assert_eq!(ds.chunks.len(), 1);
        assert_eq!(ds.chunks[0].size_bytes, 0);
        Ok(())
    }

    #[tokio::test]
    async fn register_missing_file_fails() {
        let dir = tempfile::TempDir::new().unwrap();
        let mgr = StorageManager::new_in(dir.path().to_path_buf()).await.unwrap();
        assert!(mgr
            .register_file("nope".to_string(), &dir.path().join("nope.bin"))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn transfer_roundtrip() -> Result<()> {
        let dir = tempfile::TempDir::new()?;
        let mgr = Arc::new(StorageManager::new_in(dir.path().join("node")).await?);
        let fp = dir.path().join("ship.bin");
        let payload: Vec<u8> = (0..3_000_000u32).map(|i| (i % 251) as u8).collect();
        std::fs::write(&fp, &payload)?;
        let ds = mgr.register_file("ship".to_string(), &fp).await?;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        drop(listener);
        let srv = mgr.clone();
        let task = tokio::spawn(async move { srv.serve(port).await });
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;

        // Full auth path: server enforces the local token, client presents it.
        let tok = crate::identity::api_token();
        assert!(!tok.is_empty());
        let dest = StorageManager::fetch(
            &format!("http://127.0.0.1:{port}"),
            &ds.id,
            &dir.path().join("out"),
            &tok,
        )
        .await?;
        assert_eq!(std::fs::read(&dest)?, payload);
        // Wrong token is rejected.
        assert!(StorageManager::fetch(
            &format!("http://127.0.0.1:{port}"),
            &ds.id,
            &dir.path().join("out2"),
            "wrong-token",
        )
        .await
        .is_err());
        task.abort();
        Ok(())
    }
}
