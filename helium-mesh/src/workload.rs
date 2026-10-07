//! Phase 2 scaffolding — workloads GPU, pas encore branche aux commandes.
#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

const WORKLOADS_FILE: &str = "workloads.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workload {
    pub id: String,
    pub name: String,
    pub status: WorkloadStatus,
    pub docker_image: String,
    pub gpu_count: u32,
    pub target_node: String,
    /// S3: originating market request ("" for manual `workload run` jobs).
    #[serde(default)]
    pub request_id: String,
    /// S3: requester the job counts against for quotas ("" = unlinked).
    #[serde(default)]
    pub requester: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkloadStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Stopped,
}

impl std::fmt::Display for WorkloadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkloadStatus::Pending => write!(f, "pending"),
            WorkloadStatus::Running => write!(f, "running"),
            WorkloadStatus::Completed => write!(f, "completed"),
            WorkloadStatus::Failed => write!(f, "failed"),
            WorkloadStatus::Stopped => write!(f, "stopped"),
        }
    }
}

pub struct WorkloadManager {
    workloads: RwLock<HashMap<String, Workload>>,
    base_path: std::path::PathBuf,
}

impl WorkloadManager {
    pub async fn new() -> Result<Self> {
        let base_path = Self::helium_dir()?;
        // Q5 fix: first `workload run` on a fresh node failed writing —
        // the dir was never created (Market does create_dir_all, we didn't).
        tokio::fs::create_dir_all(&base_path)
            .await
            .context("Failed to create helium directory")?;
        let workloads = if let Ok(data) = tokio::fs::read_to_string(base_path.join(WORKLOADS_FILE)).await {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self {
            workloads: RwLock::new(workloads),
            base_path,
        })
    }

    pub async fn create_workload(
        &self,
        name: String,
        docker_image: String,
        gpu_count: u32,
        request_id: &str,
        requester: &str,
    ) -> Result<Workload> {
        let id = Uuid::new_v4().to_string();

        let workload = Workload {
            id: id.clone(),
            name,
            status: WorkloadStatus::Pending,
            docker_image,
            gpu_count,
            target_node: String::new(), // Will be assigned by scheduler
            request_id: request_id.to_string(),
            requester: requester.to_string(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            logs: vec![],
        };

        // Serialize under the write guard, write the file after releasing it:
        // save-while-locked deadlocks (RwLock is not reentrant).
        let data = {
            let mut workloads = self.workloads.write().await;
            workloads.insert(id.clone(), workload.clone());
            serde_json::to_string_pretty(&*workloads)
                .context("Failed to serialize workloads")?
        };
        tokio::fs::write(self.base_path.join(WORKLOADS_FILE), data).await
            .context("Failed to write workloads file")?;

        info!("Created workload {} ({})", id, workload.name);

        Ok(workload)
    }

    pub async fn list_workloads(&self) -> Vec<Workload> {
        let workloads = self.workloads.read().await;
        workloads.values().cloned().collect()
    }

    pub async fn get_workload(&self, id: &str) -> Option<Workload> {
        let workloads = self.workloads.read().await;
        workloads.get(id).cloned()
    }

    /// S3: non-terminal (Pending/Running) jobs counting against a requester.
    pub async fn count_active_for(&self, requester: &str) -> usize {
        use WorkloadStatus::{Pending, Running};
        let workloads = self.workloads.read().await;
        workloads
            .values()
            .filter(|w| w.requester == requester && matches!(w.status, Pending | Running))
            .count()
    }

    pub async fn stop_workload(&self, id: &str) -> Result<()> {
        let data = {
            let mut workloads = self.workloads.write().await;

            if let Some(workload) = workloads.get_mut(id) {
                workload.status = WorkloadStatus::Stopped;
            }
            serde_json::to_string_pretty(&*workloads)
                .context("Failed to serialize workloads")?
        };
        tokio::fs::write(self.base_path.join(WORKLOADS_FILE), data).await
            .context("Failed to write workloads file")?;
        info!("Stopped workload {}", id);

        Ok(())
    }

    /// S2: move a freshly matched workload to Running with a first log line.
    pub async fn mark_running(&self, id: &str, log_line: String) -> Result<()> {
        let data = {
            let mut workloads = self.workloads.write().await;

            match workloads.get_mut(id) {
                Some(workload) => {
                    workload.status = WorkloadStatus::Running;
                    workload.started_at = Some(chrono::Utc::now());
                    workload.logs.push(log_line);
                }
                None => anyhow::bail!("workload {id} not found"),
            }
            serde_json::to_string_pretty(&*workloads)
                .context("Failed to serialize workloads")?
        };
        tokio::fs::write(self.base_path.join(WORKLOADS_FILE), data).await
            .context("Failed to write workloads file")?;
        info!("Workload {} running", id);

        Ok(())
    }

    fn helium_dir() -> Result<std::path::PathBuf> {
        crate::identity::helium_dir()
    }
}
