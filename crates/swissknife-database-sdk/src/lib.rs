mod error;

pub use error::{Error, Result};

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "mongodb")]
pub mod mongodb;

#[cfg(feature = "redis")]
pub mod redis;

#[cfg(feature = "elasticsearch")]
pub mod elasticsearch;

#[cfg(feature = "supabase")]
pub mod supabase;

#[cfg(feature = "clickhouse")]
pub mod clickhouse;

#[cfg(feature = "dynamodb")]
pub mod dynamodb;

#[cfg(feature = "neo4j")]
pub mod neo4j;

#[cfg(feature = "rds")]
pub mod rds;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub affected_rows: Option<u64>,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: Option<String>,
    pub columns: Vec<ColumnInfo>,
    pub primary_key: Option<Vec<String>>,
    pub row_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub index_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QueryParams {
    pub params: Vec<serde_json::Value>,
}

impl QueryParams {
    pub fn new() -> Self {
        Self { params: Vec::new() }
    }

    pub fn bind<T: Serialize>(mut self, value: T) -> Self {
        self.params.push(serde_json::to_value(value).unwrap_or(serde_json::Value::Null));
        self
    }
}

impl Default for QueryParams {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
pub trait SqlDatabaseProvider: Send + Sync {
    async fn execute(&self, query: &str, params: &QueryParams) -> Result<QueryResult>;
    async fn query(&self, query: &str, params: &QueryParams) -> Result<QueryResult>;
    async fn list_tables(&self, schema: Option<&str>) -> Result<Vec<TableInfo>>;
    async fn describe_table(&self, table: &str, schema: Option<&str>) -> Result<TableInfo>;
    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexInfo>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct FindOptions {
    pub filter: Option<serde_json::Value>,
    pub projection: Option<Vec<String>>,
    pub sort: Option<serde_json::Value>,
    pub limit: Option<u32>,
    pub skip: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct InsertResult {
    pub inserted_id: Option<String>,
    pub inserted_ids: Vec<String>,
    pub inserted_count: u64,
}

#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub matched_count: u64,
    pub modified_count: u64,
    pub upserted_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeleteResult {
    pub deleted_count: u64,
}

#[async_trait]
pub trait DocumentDatabaseProvider: Send + Sync {
    async fn list_collections(&self, database: Option<&str>) -> Result<Vec<String>>;
    async fn find(&self, collection: &str, options: &FindOptions) -> Result<Vec<Document>>;
    async fn find_one(&self, collection: &str, filter: &serde_json::Value) -> Result<Option<Document>>;
    async fn insert_one(&self, collection: &str, document: &serde_json::Value) -> Result<InsertResult>;
    async fn insert_many(&self, collection: &str, documents: &[serde_json::Value]) -> Result<InsertResult>;
    async fn update_one(&self, collection: &str, filter: &serde_json::Value, update: &serde_json::Value, upsert: bool) -> Result<UpdateResult>;
    async fn update_many(&self, collection: &str, filter: &serde_json::Value, update: &serde_json::Value) -> Result<UpdateResult>;
    async fn delete_one(&self, collection: &str, filter: &serde_json::Value) -> Result<DeleteResult>;
    async fn delete_many(&self, collection: &str, filter: &serde_json::Value) -> Result<DeleteResult>;
    async fn aggregate(&self, collection: &str, pipeline: &[serde_json::Value]) -> Result<Vec<Document>>;
}

#[async_trait]
pub trait KeyValueProvider: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>>;
    async fn set(&self, key: &str, value: &str, ttl_seconds: Option<u64>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<bool>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn keys(&self, pattern: &str) -> Result<Vec<String>>;
    async fn ttl(&self, key: &str) -> Result<Option<i64>>;
    async fn expire(&self, key: &str, seconds: u64) -> Result<bool>;
    async fn incr(&self, key: &str) -> Result<i64>;
    async fn decr(&self, key: &str) -> Result<i64>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: String,
    pub score: f32,
    pub source: serde_json::Value,
    pub highlights: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub total: u64,
    pub took_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub query: serde_json::Value,
    pub from: Option<u32>,
    pub size: Option<u32>,
    pub sort: Option<Vec<serde_json::Value>>,
    pub highlight: Option<serde_json::Value>,
    pub aggregations: Option<serde_json::Value>,
}

#[async_trait]
pub trait SearchDatabaseProvider: Send + Sync {
    async fn list_indices(&self) -> Result<Vec<String>>;
    async fn create_index(&self, name: &str, mappings: &serde_json::Value) -> Result<()>;
    async fn delete_index(&self, name: &str) -> Result<()>;
    async fn index_document(&self, index: &str, id: Option<&str>, document: &serde_json::Value) -> Result<String>;
    async fn bulk_index(&self, index: &str, documents: &[serde_json::Value]) -> Result<u64>;
    async fn search(&self, index: &str, query: &SearchQuery) -> Result<SearchResponse>;
    async fn get_document(&self, index: &str, id: &str) -> Result<Option<serde_json::Value>>;
    async fn delete_document(&self, index: &str, id: &str) -> Result<bool>;
}
