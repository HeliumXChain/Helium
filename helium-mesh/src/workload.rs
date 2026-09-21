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

    pub async fn create_workload(&self, name: String, docker_image: String, gpu_count: u32) -> Result<Workload> {
        let id = Uuid::new_v4().to_string();

        let workload = Workload {
            id: id.clone(),
            name,
            status: WorkloadStatus::Pending,
            docker_image,
            gpu_count,
            target_node: String::new(), // Will be assigned by scheduler
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            logs: vec![],
        };

        let mut workloads = self.workloads.write().await;
        workloads.insert(id.clone(), workload.clone());

        self.save().await?;

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

    pub async fn stop_workload(&self, id: &str) -> Result<()> {
        let mut workloads = self.workloads.write().await;

        if let Some(workload) = workloads.get_mut(id) {
            workload.status = WorkloadStatus::Stopped;
            self.save().await?;
            info!("Stopped workload {}", id);
        }

        Ok(())
    }

    async fn save(&self) -> Result<()> {
        let workloads = self.workloads.read().await;
        let data = serde_json::to_string_pretty(&*workloads)
            .context("Failed to serialize workloads")?;

        tokio::fs::write(self.base_path.join(WORKLOADS_FILE), data).await
            .context("Failed to write workloads file")?;

        Ok(())
    }

    fn helium_dir() -> Result<std::path::PathBuf> {
        let home = dirs::home_dir()
            .context("Failed to determine home directory")?;
        Ok(home.join(".helium"))
    }
}
