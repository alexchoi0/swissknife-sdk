mod error;

pub use error::{Error, Result};

#[cfg(feature = "pinecone")]
pub mod pinecone;

#[cfg(feature = "qdrant")]
pub mod qdrant;

#[cfg(feature = "weaviate")]
pub mod weaviate;

#[cfg(feature = "chroma")]
pub mod chroma;

#[cfg(feature = "milvus")]
pub mod milvus;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    pub id: String,
    pub values: Vec<f32>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub sparse_values: Option<SparseVector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseVector {
    pub indices: Vec<u32>,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub id: String,
    pub score: f32,
    pub values: Option<Vec<f32>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub dimension: u32,
    pub metric: DistanceMetric,
    pub vector_count: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    pub top_k: u32,
    pub include_values: bool,
    pub include_metadata: bool,
    pub filter: Option<HashMap<String, serde_json::Value>>,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UpsertOptions {
    pub namespace: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpsertResult {
    pub upserted_count: u64,
}

#[derive(Debug, Clone, Default)]
pub struct DeleteOptions {
    pub namespace: Option<String>,
    pub delete_all: bool,
    pub filter: Option<HashMap<String, serde_json::Value>>,
}

#[async_trait]
pub trait VectorDatabaseProvider: Send + Sync {
    async fn list_collections(&self) -> Result<Vec<Collection>>;
    async fn create_collection(&self, name: &str, dimension: u32, metric: DistanceMetric) -> Result<Collection>;
    async fn delete_collection(&self, name: &str) -> Result<()>;
    async fn describe_collection(&self, name: &str) -> Result<Collection>;

    async fn upsert(&self, collection: &str, vectors: &[Vector], options: &UpsertOptions) -> Result<UpsertResult>;
    async fn query(&self, collection: &str, vector: &[f32], options: &QueryOptions) -> Result<Vec<QueryResult>>;
    async fn fetch(&self, collection: &str, ids: &[&str], namespace: Option<&str>) -> Result<Vec<Vector>>;
    async fn delete(&self, collection: &str, ids: &[&str], options: &DeleteOptions) -> Result<()>;
    async fn update(&self, collection: &str, id: &str, values: Option<&[f32]>, metadata: Option<HashMap<String, serde_json::Value>>) -> Result<()>;
}

#[async_trait]
pub trait HybridSearchProvider: Send + Sync {
    async fn hybrid_query(
        &self,
        collection: &str,
        dense_vector: &[f32],
        sparse_vector: &SparseVector,
        alpha: f32,
        options: &QueryOptions,
    ) -> Result<Vec<QueryResult>>;
}
