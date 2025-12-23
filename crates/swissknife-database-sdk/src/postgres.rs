use crate::{ColumnInfo, IndexInfo, QueryParams, QueryResult, Result, SqlDatabaseProvider, TableInfo};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct PostgresClient {
    base_url: String,
}

impl PostgresClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn request(&self, query: &str, params: &[serde_json::Value]) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();

        let body = serde_json::json!({
            "query": query,
            "params": params,
        });

        let resp = client
            .post(&format!("{}/query", self.base_url))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| crate::Error::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(crate::Error::Query(error));
        }

        resp.json().await
            .map_err(|e| crate::Error::Query(e.to_string()))
    }
}

#[async_trait]
impl SqlDatabaseProvider for PostgresClient {
    async fn execute(&self, query: &str, params: &QueryParams) -> Result<QueryResult> {
        let result = self.request(query, &params.params).await?;

        let affected_rows = result
            .get("affected_rows")
            .and_then(|v| v.as_u64());

        Ok(QueryResult {
            rows: Vec::new(),
            affected_rows,
            columns: Vec::new(),
        })
    }

    async fn query(&self, query: &str, params: &QueryParams) -> Result<QueryResult> {
        let result = self.request(query, &params.params).await?;

        let rows: Vec<HashMap<String, serde_json::Value>> = result
            .get("rows")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let columns: Vec<ColumnInfo> = result
            .get("columns")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().map(|col| {
                    ColumnInfo {
                        name: col.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                        data_type: col.get("type").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                        nullable: col.get("nullable").and_then(|n| n.as_bool()).unwrap_or(true),
                    }
                }).collect()
            })
            .unwrap_or_default();

        Ok(QueryResult {
            rows,
            affected_rows: None,
            columns,
        })
    }

    async fn list_tables(&self, schema: Option<&str>) -> Result<Vec<TableInfo>> {
        let schema_filter = schema.unwrap_or("public");
        let query = format!(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = $1"
        );

        let result = self.query(&query, &QueryParams::new().bind(schema_filter)).await?;

        let tables = result.rows.iter().filter_map(|row| {
            row.get("table_name").and_then(|v| v.as_str()).map(|name| {
                TableInfo {
                    name: name.to_string(),
                    schema: Some(schema_filter.to_string()),
                    columns: Vec::new(),
                    primary_key: None,
                    row_count: None,
                }
            })
        }).collect();

        Ok(tables)
    }

    async fn describe_table(&self, table: &str, schema: Option<&str>) -> Result<TableInfo> {
        let schema_filter = schema.unwrap_or("public");
        let query = "SELECT column_name, data_type, is_nullable FROM information_schema.columns
             WHERE table_schema = $1 AND table_name = $2";

        let result = self.query(query, &QueryParams::new().bind(schema_filter).bind(table)).await?;

        let columns = result.rows.iter().map(|row| {
            ColumnInfo {
                name: row.get("column_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                data_type: row.get("data_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                nullable: row.get("is_nullable").and_then(|v| v.as_str()).map(|v| v == "YES").unwrap_or(true),
            }
        }).collect();

        Ok(TableInfo {
            name: table.to_string(),
            schema: Some(schema_filter.to_string()),
            columns,
            primary_key: None,
            row_count: None,
        })
    }

    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexInfo>> {
        let query = "SELECT indexname, indexdef FROM pg_indexes WHERE tablename = $1";

        let result = self.query(query, &QueryParams::new().bind(table)).await?;

        let indexes = result.rows.iter().filter_map(|row| {
            row.get("indexname").and_then(|v| v.as_str()).map(|name| {
                let indexdef = row.get("indexdef").and_then(|v| v.as_str()).unwrap_or("");
                IndexInfo {
                    name: name.to_string(),
                    table: table.to_string(),
                    columns: Vec::new(),
                    unique: indexdef.contains("UNIQUE"),
                    index_type: None,
                }
            })
        }).collect();

        Ok(indexes)
    }
}
