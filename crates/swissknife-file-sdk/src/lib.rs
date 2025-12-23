mod error;

pub use error::{Error, Result};

#[cfg(feature = "sftp")]
pub mod sftp;

#[cfg(feature = "ssh")]
pub mod ssh;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_directory: bool,
    pub is_symlink: bool,
    pub permissions: Option<String>,
    pub owner: Option<String>,
    pub group: Option<String>,
    pub modified_at: Option<String>,
    pub accessed_at: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub path: String,
    pub entries: Vec<FileInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct TransferOptions {
    pub overwrite: bool,
    pub preserve_permissions: bool,
    pub preserve_timestamps: bool,
    pub recursive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub speed_bytes_per_sec: Option<u64>,
}

#[async_trait]
pub trait FileOperations: Send + Sync {
    async fn list(&self, path: &str) -> Result<DirectoryListing>;
    async fn stat(&self, path: &str) -> Result<FileInfo>;
    async fn read(&self, path: &str) -> Result<Vec<u8>>;
    async fn write(&self, path: &str, data: &[u8], options: &TransferOptions) -> Result<()>;
    async fn delete(&self, path: &str) -> Result<()>;
    async fn mkdir(&self, path: &str, recursive: bool) -> Result<()>;
    async fn rmdir(&self, path: &str, recursive: bool) -> Result<()>;
    async fn rename(&self, from: &str, to: &str) -> Result<()>;
    async fn copy(&self, from: &str, to: &str, options: &TransferOptions) -> Result<()>;
    async fn exists(&self, path: &str) -> Result<bool>;
    async fn chmod(&self, path: &str, mode: u32) -> Result<()>;
}
