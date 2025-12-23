use crate::{ColumnInfo, IndexInfo, QueryParams, QueryResult, Result, SqlDatabaseProvider, TableInfo};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct SupabaseClient {
    base_url: String,
    api_key: String,
    service_key: Option<String>,
}

impl SupabaseClient {
    pub fn new(project_ref: &str, api_key: &str) -> Self {
        Self {
            base_url: format!("https://{}.supabase.co", project_ref),
            api_key: api_key.to_string(),
            service_key: None,
        }
    }

    pub fn with_service_key(mut self, service_key: &str) -> Self {
        self.service_key = Some(service_key.to_string());
        self
    }

    pub fn from_url(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            service_key: None,
        }
    }

    async fn rest_request(&self, method: reqwest::Method, path: &str, body: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let url = format!("{}/rest/v1/{}", self.base_url, path.trim_start_matches('/'));

        let auth_key = self.service_key.as_ref().unwrap_or(&self.api_key);

        let mut request = client.request(method, &url)
            .header("apikey", &self.api_key)
            .header("Authorization", format!("Bearer {}", auth_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation");

        if let Some(b) = body {
            request = request.json(&b);
        }

        let resp = request.send().await
            .map_err(|e| crate::Error::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(crate::Error::Query(error));
        }

        resp.json().await
            .map_err(|e| crate::Error::Query(e.to_string()))
    }

    pub async fn from_table(&self, table: &str) -> Result<Vec<serde_json::Value>> {
        let result = self.rest_request(reqwest::Method::GET, table, None).await?;

        let rows = result.as_array()
            .cloned()
            .unwrap_or_default();

        Ok(rows)
    }

    pub async fn insert(&self, table: &str, data: &serde_json::Value) -> Result<serde_json::Value> {
        self.rest_request(reqwest::Method::POST, table, Some(data.clone())).await
    }

    pub async fn update(&self, table: &str, data: &serde_json::Value, filter: &str) -> Result<serde_json::Value> {
        let path = format!("{}?{}", table, filter);
        self.rest_request(reqwest::Method::PATCH, &path, Some(data.clone())).await
    }

    pub async fn delete(&self, table: &str, filter: &str) -> Result<serde_json::Value> {
        let path = format!("{}?{}", table, filter);
        self.rest_request(reqwest::Method::DELETE, &path, None).await
    }

    pub async fn rpc(&self, function: &str, params: &serde_json::Value) -> Result<serde_json::Value> {
        let path = format!("rpc/{}", function);
        self.rest_request(reqwest::Method::POST, &path, Some(params.clone())).await
    }
}

#[async_trait]
impl SqlDatabaseProvider for SupabaseClient {
    async fn execute(&self, query: &str, params: &QueryParams) -> Result<QueryResult> {
        let body = serde_json::json!({
            "query": query,
            "params": params.params,
        });

        let _result = self.rpc("execute_sql", &body).await?;

        Ok(QueryResult {
            rows: Vec::new(),
            affected_rows: Some(0),
            columns: Vec::new(),
        })
    }

    async fn query(&self, query: &str, params: &QueryParams) -> Result<QueryResult> {
        let body = serde_json::json!({
            "query": query,
            "params": params.params,
        });

        let result = self.rpc("query_sql", &body).await?;

        let rows: Vec<HashMap<String, serde_json::Value>> = result
            .as_array()
            .map(|arr| {
                arr.iter().filter_map(|v| {
                    v.as_object().map(|obj| {
                        obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(QueryResult {
            rows,
            affected_rows: None,
            columns: Vec::new(),
        })
    }

    async fn list_tables(&self, schema: Option<&str>) -> Result<Vec<TableInfo>> {
        let schema_name = schema.unwrap_or("public");

        let body = serde_json::json!({
            "schema": schema_name,
        });

        let result = self.rpc("get_tables", &body).await.unwrap_or(serde_json::Value::Array(Vec::new()));

        let tables = result
            .as_array()
            .map(|arr| {
                arr.iter().filter_map(|v| {
                    v.get("table_name").and_then(|n| n.as_str()).map(|name| {
                        TableInfo {
                            name: name.to_string(),
                            schema: Some(schema_name.to_string()),
                            columns: Vec::new(),
                            primary_key: None,
                            row_count: None,
                        }
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(tables)
    }

    async fn describe_table(&self, table: &str, schema: Option<&str>) -> Result<TableInfo> {
        let schema_name = schema.unwrap_or("public");

        let body = serde_json::json!({
            "table": table,
            "schema": schema_name,
        });

        let result = self.rpc("get_table_columns", &body).await.unwrap_or(serde_json::Value::Array(Vec::new()));

        let columns = result
            .as_array()
            .map(|arr| {
                arr.iter().map(|v| {
                    ColumnInfo {
                        name: v.get("column_name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                        data_type: v.get("data_type").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                        nullable: v.get("is_nullable").and_then(|n| n.as_str()).map(|s| s == "YES").unwrap_or(true),
                    }
                }).collect()
            })
            .unwrap_or_default();

        Ok(TableInfo {
            name: table.to_string(),
            schema: Some(schema_name.to_string()),
            columns,
            primary_key: None,
            row_count: None,
        })
    }

    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexInfo>> {
        let body = serde_json::json!({
            "table": table,
        });

        let result = self.rpc("get_table_indexes", &body).await.unwrap_or(serde_json::Value::Array(Vec::new()));

        let indexes = result
            .as_array()
            .map(|arr| {
                arr.iter().filter_map(|v| {
                    v.get("index_name").and_then(|n| n.as_str()).map(|name| {
                        IndexInfo {
                            name: name.to_string(),
                            table: table.to_string(),
                            columns: Vec::new(),
                            unique: v.get("is_unique").and_then(|u| u.as_bool()).unwrap_or(false),
                            index_type: v.get("index_type").and_then(|t| t.as_str()).map(String::from),
                        }
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(indexes)
    }
}
