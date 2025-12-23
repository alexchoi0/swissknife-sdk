mod error;

pub use error::{Error, Result};

#[cfg(feature = "s3")]
pub mod s3;

#[cfg(feature = "gcs")]
pub mod gcs;

#[cfg(feature = "azure")]
pub mod azure;

#[cfg(feature = "dropbox")]
pub mod dropbox;

#[cfg(feature = "cloudflare")]
pub mod cloudflare;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub name: String,
    pub region: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub key: String,
    pub bucket: String,
    pub size: u64,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub storage_class: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectVersion {
    pub key: String,
    pub version_id: String,
    pub is_latest: bool,
    pub last_modified: DateTime<Utc>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    pub key: String,
    pub bucket: String,
    pub etag: Option<String>,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyResult {
    pub key: String,
    pub bucket: String,
    pub etag: Option<String>,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: Option<u32>,
    pub continuation_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListResult {
    pub objects: Vec<Object>,
    pub common_prefixes: Vec<String>,
    pub is_truncated: bool,
    pub next_continuation_token: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UploadOptions {
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub cache_control: Option<String>,
    pub content_disposition: Option<String>,
    pub metadata: HashMap<String, String>,
    pub storage_class: Option<String>,
    pub acl: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PresignedUrl {
    pub url: String,
    pub method: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct PresignOptions {
    pub expires_in_seconds: u64,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipartUpload {
    pub upload_id: String,
    pub key: String,
    pub bucket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadPart {
    pub part_number: u32,
    pub etag: String,
}

#[async_trait]
pub trait ObjectStorageProvider: Send + Sync {
    async fn list_buckets(&self) -> Result<Vec<Bucket>>;
    async fn create_bucket(&self, name: &str, region: Option<&str>) -> Result<Bucket>;
    async fn delete_bucket(&self, name: &str) -> Result<()>;
    async fn bucket_exists(&self, name: &str) -> Result<bool>;

    async fn list_objects(&self, bucket: &str, options: &ListOptions) -> Result<ListResult>;
    async fn get_object(&self, bucket: &str, key: &str) -> Result<Vec<u8>>;
    async fn get_object_metadata(&self, bucket: &str, key: &str) -> Result<Object>;
    async fn put_object(&self, bucket: &str, key: &str, data: &[u8], options: &UploadOptions) -> Result<UploadResult>;
    async fn delete_object(&self, bucket: &str, key: &str) -> Result<()>;
    async fn delete_objects(&self, bucket: &str, keys: &[&str]) -> Result<Vec<String>>;
    async fn copy_object(&self, source_bucket: &str, source_key: &str, dest_bucket: &str, dest_key: &str) -> Result<CopyResult>;
    async fn object_exists(&self, bucket: &str, key: &str) -> Result<bool>;
}

#[async_trait]
pub trait PresignedUrlProvider: Send + Sync {
    async fn generate_presigned_get_url(&self, bucket: &str, key: &str, options: &PresignOptions) -> Result<PresignedUrl>;
    async fn generate_presigned_put_url(&self, bucket: &str, key: &str, options: &PresignOptions) -> Result<PresignedUrl>;
}

#[async_trait]
pub trait MultipartUploadProvider: Send + Sync {
    async fn create_multipart_upload(&self, bucket: &str, key: &str, options: &UploadOptions) -> Result<MultipartUpload>;
    async fn upload_part(&self, bucket: &str, key: &str, upload_id: &str, part_number: u32, data: &[u8]) -> Result<UploadPart>;
    async fn complete_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str, parts: &[UploadPart]) -> Result<UploadResult>;
    async fn abort_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<()>;
}

#[async_trait]
pub trait VersioningProvider: Send + Sync {
    async fn list_object_versions(&self, bucket: &str, key: &str) -> Result<Vec<ObjectVersion>>;
    async fn get_object_version(&self, bucket: &str, key: &str, version_id: &str) -> Result<Vec<u8>>;
    async fn delete_object_version(&self, bucket: &str, key: &str, version_id: &str) -> Result<()>;
}
