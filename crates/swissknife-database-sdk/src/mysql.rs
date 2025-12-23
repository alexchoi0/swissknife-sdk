use crate::{ColumnInfo, IndexInfo, QueryParams, QueryResult, Result, SqlDatabaseProvider, TableInfo};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct MySqlClient {
    connection_string: String,
}

impl MySqlClient {
    pub fn new(connection_string: &str) -> Self {
        Self {
            connection_string: connection_string.to_string(),
        }
    }

    pub fn connection_string(&self) -> &str {
        &self.connection_string
    }
}

#[async_trait]
impl SqlDatabaseProvider for MySqlClient {
    async fn execute(&self, query: &str, params: &QueryParams) -> Result<QueryResult> {
        let client = reqwest::Client::new();

        let body = serde_json::json!({
            "query": query,
            "params": params.params,
        });

        let resp = client
            .post(&self.connection_string)
            .json(&body)
            .send()
            .await
            .map_err(|e| crate::Error::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(crate::Error::Query(error));
        }

        let result: serde_json::Value = resp.json().await
            .map_err(|e| crate::Error::Query(e.to_string()))?;

        Ok(QueryResult {
            rows: Vec::new(),
            affected_rows: result.get("affected_rows").and_then(|v| v.as_u64()),
            columns: Vec::new(),
        })
    }

    async fn query(&self, query: &str, params: &QueryParams) -> Result<QueryResult> {
        let client = reqwest::Client::new();

        let body = serde_json::json!({
            "query": query,
            "params": params.params,
        });

        let resp = client
            .post(&self.connection_string)
            .json(&body)
            .send()
            .await
            .map_err(|e| crate::Error::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(crate::Error::Query(error));
        }

        let result: serde_json::Value = resp.json().await
            .map_err(|e| crate::Error::Query(e.to_string()))?;

        let rows: Vec<HashMap<String, serde_json::Value>> = result
            .get("rows")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let columns: Vec<ColumnInfo> = result
            .get("columns")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        Ok(QueryResult {
            rows,
            affected_rows: None,
            columns,
        })
    }

    async fn list_tables(&self, schema: Option<&str>) -> Result<Vec<TableInfo>> {
        let db = schema.unwrap_or("information_schema");
        let query = format!(
            "SELECT TABLE_NAME FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = '{}'",
            db
        );
        let result = self.query(&query, &QueryParams::new()).await?;

        let tables = result.rows.iter().filter_map(|row| {
            row.get("TABLE_NAME").and_then(|v| v.as_str()).map(|name| {
                TableInfo {
                    name: name.to_string(),
                    schema: Some(db.to_string()),
                    columns: Vec::new(),
                    primary_key: None,
                    row_count: None,
                }
            })
        }).collect();

        Ok(tables)
    }

    async fn describe_table(&self, table: &str, schema: Option<&str>) -> Result<TableInfo> {
        let db = schema.unwrap_or("information_schema");
        let query = format!(
            "SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE FROM INFORMATION_SCHEMA.COLUMNS
             WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}'",
            db, table
        );

        let result = self.query(&query, &QueryParams::new()).await?;

        let columns = result.rows.iter().map(|row| {
            ColumnInfo {
                name: row.get("COLUMN_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                data_type: row.get("DATA_TYPE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                nullable: row.get("IS_NULLABLE").and_then(|v| v.as_str()).map(|v| v == "YES").unwrap_or(true),
            }
        }).collect();

        Ok(TableInfo {
            name: table.to_string(),
            schema: Some(db.to_string()),
            columns,
            primary_key: None,
            row_count: None,
        })
    }

    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexInfo>> {
        let query = format!("SHOW INDEX FROM {}", table);
        let result = self.query(&query, &QueryParams::new()).await?;

        let indexes = result.rows.iter().filter_map(|row| {
            row.get("Key_name").and_then(|v| v.as_str()).map(|name| {
                IndexInfo {
                    name: name.to_string(),
                    table: table.to_string(),
                    columns: Vec::new(),
                    unique: row.get("Non_unique").and_then(|v| v.as_i64()).map(|v| v == 0).unwrap_or(false),
                    index_type: row.get("Index_type").and_then(|v| v.as_str()).map(String::from),
                }
            })
        }).collect();

        Ok(indexes)
    }
}
